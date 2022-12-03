import { Client } from "twitter-api-sdk";
import { config } from "dotenv";
import { OAuth2User } from "twitter-api-sdk/dist/OAuth2User";
import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { homedir } from "node:os";
import express from "express";
import { randomInt } from "node:crypto";
import { Server } from "node:http";
import { inspect } from "node:util"
import { open } from "sqlite";
import sqlite3 from "sqlite3";
import { getUsersIdBookmarks, TwitterPaginatedResponse, TwitterParams, TwitterResponse, usersIdLikedTweets } from "twitter-api-sdk/dist/types";

config();

const tweetParams: TwitterParams<getUsersIdBookmarks | usersIdLikedTweets> = {
    "tweet.fields": [
        "attachments",
        "text",
        "author_id",
        "referenced_tweets",
        "created_at",
    ],
    "expansions": [
        "attachments.media_keys",
        "author_id",
        "referenced_tweets.id",
        "referenced_tweets.id.author_id",
    ],
    "media.fields": [
        "alt_text",
        "media_key",
        "type",
        "url",
        "variants",
        "width",
        "height"
    ],
};

// Mmmmmmmmmmmmm nodejs
(async () => {
    const authClient = new OAuth2User({
        client_id: process.env.CLIENT_ID as string,
        client_secret: process.env.CLIENT_SECRET as string,
        scopes: [
            "bookmark.read",
            "like.read",
            "list.read",
            "users.read",
            "offline.access",
            "tweet.read",
        ],
        callback: "http://localhost:3621/callback"
    });

    const client = new Client(authClient);
    const harrowDir = join(homedir(), ".local/share/harrow-downloader/");

    const db = await open({
        filename: join(harrowDir, "db.sqlite"),
        driver: sqlite3.Database
    });

    let s: Server | undefined;
    const prom = new Promise<void>(async (resolve, _) => {
        try {
            authClient.token = JSON.parse(await readFile(join(harrowDir, "auth.json"), "ascii"));
            await authClient.refreshAccessToken();
            resolve();
        } catch {
            // Not authenticated, login
            const app = express();

            const stateOrig = randomInt(10000);

            app.get("/callback", async (req, res) => {
                const { code, state } = req.query;
                if (state != stateOrig.toString()) {
                    return res.status(500).send("Invalid state");
                }

                await authClient.requestAccessToken(code as string);
                res.send("You can close the tab now");
                resolve();
            });

            app.get("/login", async (req, res) => {
                res.redirect(authClient.generateAuthURL({
                    state: stateOrig.toString(),
                    code_challenge_method: "s256"
                }));
            });

            s = app.listen(3621, () => {
                console.log("Go to http://localhost:3621/login to login");
            });
        }
    });

    await prom;
    s?.close();

    await writeFile(join(harrowDir, "auth.json"), JSON.stringify(authClient.token));

    const userId = (await client.users.findMyUser()).data?.id ?? "";

    let tweetData = await client.tweets.usersIdLikedTweets(userId, { ...tweetParams });

    let count = tweetData.meta?.result_count ?? 0;
    let errorCount = 0;

    // Liked tweets
    do {
        if (tweetData.data == undefined) continue;

        for (let post of tweetData.data) {
            // Insert the post
            try {
                await db.run(
                    "INSERT INTO post (id, account_username, text) VALUES (?, ?, ?)",
                    post.id,
                    tweetData.includes?.users?.find((u) => u.id == post.author_id)?.username,
                    post.text
                );
            } catch {
                errorCount++;
            }

            // Insert likes if it does not already exist
            try {
                if (await db.get("SELECT post_id FROM likes WHERE post_id = ?", post.id) == undefined) {
                    await db.run("INSERT INTO likes (post_id) VALUES (?)", post.id);
                }
            } catch {
                errorCount++;
            }

            // Insert media
            if (post.attachments?.media_keys) {
                for (let attach of post.attachments.media_keys) {
                    const media: any = tweetData.includes?.media?.find((m) => m.media_key == attach);
                    if (media?.type == "photo") {
                        try {
                            await db.run(
                                "INSERT INTO media (id, url, alt_text, type, bitrate, post_id) VALUES (?, ?, ?, ?, ?, ?)",
                                media.media_key,
                                media.url,
                                media.alt_text,
                                "photo",
                                0,
                                post.id
                            );
                        } catch {
                            errorCount++;
                        }
                    } else if (media?.type == "animated_gif" || media?.type == "video") {
                        for (let variant of media?.variants) {
                            try {
                                await db.run(
                                    "INSERT INTO media (id, url, alt_text, type, bitrate, post_id) VALUES (?, ?, ?, ?, ?, ?)",
                                    media.media_key,
                                    variant.url,
                                    media.alt_text,
                                    media.type,
                                    variant.bit_rate ?? 0,
                                    post.id
                                );
                            } catch {
                                errorCount++;
                            }
                        }
                    }
                }
            }
        }

        tweetData = await client.tweets.usersIdLikedTweets(userId, { ...tweetParams, pagination_token: tweetData.meta?.next_token });
        console.log(inspect(tweetData, true, null, true));
        count += tweetData.meta?.result_count ?? 0;
    } while (tweetData.meta?.next_token != undefined);

    let tweetBookmarkedData = await client.bookmarks.getUsersIdBookmarks(userId, { ...tweetParams });
    count += tweetBookmarkedData.meta?.result_count ?? 0;

    // Bookmarked tweets
    do {
        if (tweetBookmarkedData.data == undefined) continue;

        for (let post of tweetBookmarkedData.data) {
            // Insert the post
            try {
                await db.run(
                    "INSERT INTO post (id, account_username, text) VALUES (?, ?, ?)",
                    post.id,
                    tweetBookmarkedData.includes?.users?.find((u) => u.id == post.author_id)?.username,
                    post.text
                );
            } catch {
                errorCount++;
            }

            // Insert bookmarks if it does not already exist
            try {
                if (await db.get("SELECT post_id FROM bookmarks WHERE post_id = ?", post.id) == undefined) {
                    await db.run("INSERT INTO bookmarks (post_id) VALUES (?)", post.id);
                }
            } catch {
                errorCount++;
            }

            // Insert media
            if (post.attachments?.media_keys) {
                for (let attach of post.attachments.media_keys) {
                    const media: any = tweetBookmarkedData.includes?.media?.find((m) => m.media_key == attach);
                    if (media?.type == "photo") {
                        try {
                            await db.run(
                                "INSERT INTO media (id, url, alt_text, type, bitrate, post_id) VALUES (?, ?, ?, ?, ?, ?)",
                                media.media_key,
                                media.url,
                                media.alt_text,
                                "photo",
                                0,
                                post.id
                            );
                        } catch {
                            errorCount++;
                        }
                    } else if (media?.type == "animated_gif" || media?.type == "video") {
                        for (let variant of media?.variants) {
                            try {
                                await db.run(
                                    "INSERT INTO media (id, url, alt_text, type, bitrate, post_id) VALUES (?, ?, ?, ?, ?, ?)",
                                    media.media_key,
                                    variant.url,
                                    media.alt_text,
                                    media.type,
                                    variant.bit_rate ?? 0,
                                    post.id
                                );
                            } catch {
                                errorCount++;
                            }
                        }
                    }
                }
            }
        }

        tweetBookmarkedData = await client.bookmarks.getUsersIdBookmarks(userId, { ...tweetParams, pagination_token: tweetBookmarkedData.meta?.next_token });
        console.log(inspect(tweetBookmarkedData, true, null, true));
        count += tweetBookmarkedData.meta?.result_count ?? 0;
    } while (tweetBookmarkedData.meta?.next_token != undefined);

    console.log(`Got ${count} requests`);
    console.log("Final error count", errorCount);
})().catch((err) => {
    console.log(err.error)
}).then();
