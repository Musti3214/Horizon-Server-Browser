# GitHub Pages and domain setup

Intended repository: `Musti3214/Horizon-Server-Browser`.
Free project address: `https://musti3214.github.io/Horizon-Server-Browser/`.
This is a free GitHub subdomain with HTTPS, not a separately registered domain.

Create a public repository, then choose **Settings → Pages → Source → GitHub Actions**.
Push this checkout to `main`. The Pages workflow validates the package and website,
builds the project-path version and deploys it. Inspect the Actions result before
claiming the new site is live. The initial repository creation and Pages setting
need an account with repository administration permission.

For local builds, the defaults target that project path:

```sh
python tools/validate_public.py
python website/build_site.py
python website/tools/validate_site.py
python -m http.server 4174 --bind 127.0.0.1 --directory preview
```

For the preview command, first copy `website/dist/` into
`preview/Horizon-Server-Browser/`. This reproduces the actual GitHub URL prefix.

`SITE_ORIGIN` is the HTTPS origin without a trailing slash and `SITE_BASE_PATH`
is either the repository path or empty for a later custom-domain root. The builder
uses these for internal links, scripts, catalog fetches, canonical URLs, Open Graph,
structured data and the sitemap. Do not set a custom CNAME until the domain is owned
and its DNS is correctly configured.

After verifying the new deployment, make the older website point to this canonical
location rather than maintaining two competing canonical origins. Keep the older
site working until the new website, images, downloads and links have been checked.

For Google, the site includes accessible HTML content, page descriptions, useful
server-finding text, image descriptions, sitemap and robots.txt. No keyword stuffing,
fabricated reviews or ranking promises. Google says domain keywords and generic
TLD choice have little ranking effect: [SEO Starter Guide](https://developers.google.com/search/docs/fundamentals/seo-starter-guide).
Indexing and Search Console verification still depend on the actual published site
and account ownership.
