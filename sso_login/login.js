const puppeteer = require("puppeteer");
const fs = require("fs");
const { exit } = require("process");

const sso_login_url = null; /* ADD SSO LOGIN URL HERE */

const cookieNamePrefix = "Maconomy-";
const cookie_file_name = "maconomy_cookies";
const timeout = 5 * 60_000;

/**
 * @param {puppeteer.HTTPResponse} response
 * @param {string} withNamePrefix
 * @returns {{name: string, value: string}}
 */
function get_cookie(response, withNamePrefix) {
  const cookiesString = response.headers()["set-cookie"] || "";
  return cookiesString
    .split("\n")
    .map((c) => {
      console.log(`c = '${c}'`);
      const splitIndex = c.indexOf("=");
      const name = c.slice(0, splitIndex);
      const value = c.slice(splitIndex + 1);
      return { name, value };
    })
    .find(({ name }) => name.startsWith(withNamePrefix));
}

(async () => {
  const browser = await puppeteer.launch({ headless: false });
  const page = await browser.newPage();

  await page.goto(sso_login_url);

  let auth_cookie;

  await page
    .waitForResponse(
      (response) => {
        auth_cookie = get_cookie(response, cookieNamePrefix);
        return auth_cookie;
      },
      { timeout },
    )
    .catch((err) => {
      if (err instanceof puppeteer.TimeoutError) {
        console.log("Timed out waiting for user to sign in.");
      } else {
        console.error(`Unknown error occurred: ${err}`);
      }
      exit(1);
    });

  const cookie_string = `${auth_cookie.name} ${auth_cookie.value}`;

  fs.writeFile(cookie_file_name, cookie_string, (err) => {
    if (err) {
      console.error(`Failed to write to ${cookie_file_name}: ${err}`);
      exit(1);
    }
  });

  await browser.close();

  exit(0);
})();
