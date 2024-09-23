const puppeteer = require("puppeteer");
const fs = require("fs");
const { exit } = require("process");

const sso_login_url = null; /* ADD SSO LOGIN URL HERE */
const cookie_file_name = "maconomy_cookies";

(async () => {
  const browser = await puppeteer.launch({ headless: false });
  const page = await browser.newPage();

  await page.goto(sso_login_url);

  const msg =
    "Please complete the login process manually. Once logged in, press Enter to capture cookies.";
  console.log(msg);

  await new Promise((resolve) => process.stdin.once("data", resolve));

  const cookies = await page.cookies();
  const auth_cookie = cookies.find((c) => c.name.startsWith("Maconomy-"));

  if (!auth_cookie) {
    console.error("No authentication cookie found");
    exit(1);
  }

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
