# Data and network behavior

## Personal server data

Saved servers, favorites, groups, tags, notes, settings and filter profiles live
in the local game profile. Existing `servers.dat` entries and unknown fields are
preserved. The mod does not upload the player's saved server list.

## Public discovery

Online discovery starts disabled, including for users upgrading from 2.2.0. It
starts after the player confirms the in-game privacy screen. Requests go to
minecraftserve.rs and api.modrinth.com. Each source has
a separate five-minute cache and a persistent limit of one request per minute.
Search text, player names, personal addresses and notes are not sent to these
directory APIs. Modrinth supplies public addresses, categories, language and region;
its status metadata is not treated as a fresh direct ping. Modpacks and listing
images are not downloaded or installed.

Discover sends direct status pings to visible rows and a small margin. Scrolling
updates that window and automatic refresh checks the window or the saved list.
Those operators can observe the connecting IP and the request. Authentication
and joining use Horizon's existing account/network implementation.

## Feedback

The native edition saves feedback and server-proposal drafts locally. The toolbar
Ko-fi button opens `https://ko-fi.com/foryou41` in the default browser.

## Website

The website's own code has no analytics, advertising scripts or external fonts.
Search and filters run in the browser. The full catalog page stores a shareable
search in the URL; that URL can appear in hosting request logs when opened.
Images, scripts and the dated address catalog are fetched from the site host.
Copying an address uses the browser clipboard. The website does not ping servers.

GitHub Pages logs visitors' IP addresses for security purposes, including visitors
who are not signed into GitHub. See [GitHub Pages data collection](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages#data-collection).
External server-directory links have their own terms and data practices.
