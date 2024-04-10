const url = require("url");
const https = require("https");
const fs = require("fs");
const { default: axios } = require("axios");

const hostname = "localhost";
const port = 8080;
const clientId = "b9c703b4-7034-46da-adce-9cbfeba8a5de";
const codeChallenge = "4RK3kYNSf8d2zEwy8KgYl-yY6aMXU4vf6SIabXiMUmg";
const codeVerifier = "VrKfQL3XMJAMI15TufQB9mkHTvnEx2mcNcFgqwi-LZs";

const serverOptions = {
  key: fs.readFileSync("certificate/private-key.pem"),
  cert: fs.readFileSync("certificate/certificate.pem"),
};

const server = https.createServer(serverOptions, async (req, res) => {
  console.log("req.headers = ", req.headers);
  console.log("req = ", req);

  const code = url.parse(req.url, true).query.code;
  if (code === undefined) {
    console.error("No code");
    return;
  }

  tokenUrl =
    "https://login.microsoftonline.com/3b68c6c1-04d4-4e86-875f-e48fa80b9529/oauth2/token";
  const tokenOptions = {
    grant_type: "authorization_code",
    client_id: clientId,
    // client_secret: CLIENT_SECRET,
    code_verifier: codeVerifier,
    code,
    redirect_uri: `https://${hostname}:${port}`,
  };

  const body = new URLSearchParams(tokenOptions).toString();
  const options = {
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
  };
  axios
    .post(tokenUrl, body, options)
    .then((response) => {
      console.log(response.data);
      // Handle successful response here
    })
    .catch((error) => {
      console.error(error.response.status);
      console.error(error.response.data);
      // Handle error here
    });

  res.end("Hello, World! This is HTTPS!\n");
});

// Start listening on the defined port and hostname
server.listen(port, hostname, () => {
  console.log(`Server running at https://${hostname}:${port}/`);
});

// https://login.microsoftonline.com/3b68c6c1-04d4-4e86-875f-e48fa80b9529/oauth2/authorize?client_id=b9c703b4-7034-46da-adce-9cbfeba8a5de&response_type=code&state=vG79KBTlZGzftCOTAg5GUw&code_challenge=4RK3kYNSf8d2zEwy8KgYl-yY6aMXU4vf6SIabXiMUmg&code_challenge_method=S256&redirect_uri=https%3a%2f%2flocalhost%3a8080%0a&scope=openid

// TODO: is it this request?
// https://login.microsoftonline.com/3b68c6c1-04d4-4e86-875f-e48fa80b9529/oauth2/token
