const url = require("url");
const https = require("https");
const fs = require("fs");
const { default: axios } = require("axios");

const hostname = "localhost";
const port = 8080;
const clientId = "b9c703b4-7034-46da-adce-9cbfeba8a5de";
const codeVerifier = "VrKfQL3XMJAMI15TufQB9mkHTvnEx2mcNcFgqwi-LZs";
const codeChallenge = "4RK3kYNSf8d2zEwy8KgYl-yY6aMXU4vf6SIabXiMUmg";

const serverOptions = {
  key: fs.readFileSync("certificate/private-key.pem"),
  cert: fs.readFileSync("certificate/certificate.pem"),
};

const server = https.createServer(serverOptions, async (req, res) => {
  const code = url.parse(req.url, true).query.code;
  if (code === undefined) {
    return;
  }

  tokenUrl =
    "https://login.microsoftonline.com/3b68c6c1-04d4-4e86-875f-e48fa80b9529/oauth2/token";
  const body = new URLSearchParams({
    grant_type: "authorization_code",
    client_id: clientId,
    // client_secret: "foo", // This shouldn't be needed?
    code_verifier: codeVerifier,
    code,
    redirect_uri: `https://${hostname}:${port}`,
  }).toString();

  const options = {
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
  };
  axios
    .post(tokenUrl, body, options)
    .then((response) => {
      console.log(response.status);
      console.log(response.data);
    })
    .catch((error) => {
      console.error(error.response.status);
      console.error(error.response.data);
    });

  res.end("Hello, World! This is HTTPS!\n");
});

// Start listening on the defined port and hostname
server.listen(port, hostname, () => {
  console.log(`Server running at https://${hostname}:${port}/`);
});

// https://login.microsoftonline.com/3b68c6c1-04d4-4e86-875f-e48fa80b9529/oauth2/authorize?client_id=b9c703b4-7034-46da-adce-9cbfeba8a5de&response_type=code&code_challenge=4RK3kYNSf8d2zEwy8KgYl-yY6aMXU4vf6SIabXiMUmg&code_challenge_method=S256&redirect_uri=https%3a%2f%2flocalhost%3a8080%0a&scope=openid
