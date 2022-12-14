# Harrow Downloader

A small twitter utility for downloading/archiving images and videos from
the user's likes and bookmarks.

# Installation

Currently due to nodejs being _the worst_ the utility must be ran from source.

# Building

## Requirements

To build the utility the following additional tools must be installed:

- Latest [rust compiler](https://www.rust-lang.org/) with cargo (recommended to install through [rustup](https://rustup.rs/))
- Latest [node runtime](https://nodejs.org/)
- [pnpm](https://pnpm.io/) installed globally (optional but recommended)

### Twitter API Key

Because the project is run from source you have to manually obtain a developer key from [twitter](https://developer.twitter.com/)
and do the following:

- Create a new app (see the getting started pages on how to create a new app), the name is irrelevant although it must be unique
- Save the credentials you are given somewhere safe
- In the developer portal click on your newly created app and go to `User authentication settings`
- Set the `App permissions` to `Read` (or whatever else I'm not your mom)
- Crucially set the `Type of App` to `Native App`
- Add a callback url to `http://localhost:3621/callback` in the `App Info` section
- Save the changes

Afterwards you should be given a few "client" tokens (SAVE THESE). After you obtain the client tokens create a new file called
`.env` in the `db-gen` directory and put the following contents in it (replacing the filler text with the relevant tokens, excluding the less
than and greater than symbols):

```env
CLIENT_ID=<your client ID token>
CLIENT_SECRET=<your client secret>
```

After that you should be all set.


## Building

Clone the project

```sh
git clone https://github.com/System-rat/harrow-downloader
cd harrow-downloader
```

After all the tools are installed run the following commands:

```sh
cd db-gen
# Install all the required packages
pnpm install
# Build the database generator
pnpm build
cd ..
```

| NOTE: If you get an error that a file already exists just run `rm -f lib/*` or the other
OS equivalents to remove the dist files

After that you should be able to build and run the CLI through cargo:

```sh
cargo run --release
```

# Usage

During the first-time run of the CLI you will be prompted to login to your Twitter account
(This is to allow searching through likes/bookmarks if the account is private). Afterwards
the credentials are cached for multiple uses.

The CLI will first get a list of all the tweets from your liked/bookmarked tweets (this might take a while)
after which it will go through all the media links and download the highest-resolution media file available.

After the download is complete some optional (enabled by default) organizational folders will be made with symbolic
links to make browsing through the images by author, likes, and bookmarks easier.

Optionally (enabled by default) a `.txt` file will be generated for every post with the post's text, author and related
files. The filename will be comprised of all the images/videos that are within the post with a `__` (double underscore)
as the separator.

Run the help command for additional options:

```sh
cargo run --release -- --help
```

# License
This project is licensed under the MIT license.
