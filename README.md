# ic-docutrack

### First Milestone

Extending DocuTrack proof-of-concept dapp for sharing and managing documents. You can upload documents and authorize people to access them with a few clicks. Access to shared documents can expire or be explicitly revoked. In addition, you can ask other people to upload documents for you by simply sharing a link (no login required). Documents are transmitted and stored in encrypted form. The dapp can be used with any standard web browser, no plugins or extensions are needed and no passwords need to be remembered.

Such a dapp can only be realized on the Internet Computer (IC). It is the only blockchain network that can serve web content directly. Furthermore, its programming model enables such complex applications with privacy-preserving identity management fully on-chain. Last but not least, the IC provides low latency, efficiency and affordable storage facilities.

The documents are encrypted at all times, so no one–including IC node providers– but designated users can decrypt them.
In more detail, DocuTrack demonstrates how a user Alice who created an account with the dapp (using Internet Identity) can ask a Bob to upload documents for her without having to create an account himself.
This feature makes it very easy and secure for service providers (e.g., a client advisor or wealth manager) to request documents of any type (e.g., house deeds, passport pictures or log files) from clients or other third parties.
Other document sharing apps require users to exchange public keys or other cryptographic material with which people typically struggle a lot.

You can try out DocuTrack on [https://l7rii-2yaaa-aaaao-a4liq-cai.icp0.io/](https://l7rii-2yaaa-aaaao-a4liq-cai.icp0.io/).

The dapp is explained in more detail [in the documentation](./docs/README.md).

## Disclaimer: please read carefully

This is a proof of concept dapp that demonstrates the potential of building confidential document management on the IC. Do not use this code as is for sensitive data as there are the following issues that should be addressed before the dapp can be considered production-ready:

- Users may lose their notes if they accidentally clean the browser data (localStorage)
- The frontend re-uses the generated public- and private-key pair for every identity in the same browser. In a better implementation, this key pair should be unique per principal and not managed by the browser at all.
- The same user cannot access the docs in another browser because the decryption key will not be available there.
- Lack of key update: Given that the key used to encrypted the files is never refreshed, the privacy of the data is no longer guaranteed if an attacker learns this key (for instance, by corrupting the local storage of one of the users).

The best solution for the first three bullet points is to apply [vetKeys](https://internetcomputer.org/blog/features/vetkey-primer/) to ensure in a clean and robust way that the same key pair can be extracted for each principal, regardless of the machine and browser used to access the dapp. Until this feature is available, key management could be implemented with WebAuthn extensions. However, these approaches are probably rather brittle, due to lacking widespread support in browsers and HW. For the last point, key revocation and/or key rotation should be used.

### Next Iterations will address this disclaimer points

## Development

### Prerequisites

- [Rust (1.85 or later)](https://rustup.rs/): to build the canisters
- [Node.js](https://nodejs.org/en/) (v22.0.0 or later)
- [DFX](https://internetcomputer.org/docs/building-apps/getting-started/install) (v0.23 or later)
  - **NNS** extension is required to deploy the canisters, so make sure to install it with `dfx extension install nns`
- [Just](https://just.systems/) to run scripts
- [ic-wasm](https://github.com/dfinity/ic-wasm): to bundle the canisters
- [candid-extractor](https://github.com/dfinity/candid-extractor): to extract the candid interface of the canisters

### Build canisters

Just run the following command to build all canisters:

```sh
just build_all_canisters
```

### Fetch External Canisters

Just run the following command to fetch external deps canisters:
Fetching external cansiters must be made after build command.

```sh
just fetch_all_canisters
```

### Test canisters

To run the tests, run the following command:

```sh
just test [test_name]
just integration_test [test_name]
```

Integration tests depend on external canisters, fetch them first.

### Lint and format

```sh
just clippy
just fmt_nightly
```

### Run canisters locally

To run the dapp locally, run the following in one terminal window:

```sh
just dfx_deploy_local install
```

This will eventually print the URL of the dapp, which you can open in your browser:

```txt
webserver-port: 8080
frontend: uzt4z-lp777-77774-qaabq-cai
marketing: u6s2n-gx777-77774-qaaba-cai
orbit-station: umunu-kh777-77774-qaaca-cai
orchestrator: uxrrr-q7777-77774-qaaaq-cai
frontend url: http://uzt4z-lp777-77774-qaabq-cai.raw.localhost:8080
marketing url: http://u6s2n-gx777-77774-qaaba-cai.raw.localhost:8080
```

To stop the DFX local replica, run:

```sh
just dfx_stop
```

## Local frontend development

After deploying locally both Internet Identity and the backend canister, you can run the dapp frontend and the home page development server.

### Frontend project of the dapp

```sh
pnpm --filter frontend run dev
```

### Home page

```sh
pnpm --filter landing-page run dev
```
