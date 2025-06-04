# Docutrack

DocuTrack is a dapp for sharing and managing documents. You can upload documents and authorize people to access them with a few mouse clicks.

Access to shared documents can expire or be explicitly revoked. In addition, you can ask other people to upload documents for you by simply sharing a link (no login required).

Documents are transmitted and stored in encrypted form. The dapp can be used with any standard
web browser, no plugins or extensions are needed and no passwords need to be remembered.

Such a dapp can only be realized on the Internet Computer (IC). It is the only blockchain network that can serve web content directly.

Furthermore, its programming model enables such complex applications with privacy-preserving identity management fully on-chain.

Last but not least, the IC provides low latency, efficiency and affordable storage facilities.

The documents are encrypted at all times, so no one–including IC node providers– but designated users can decrypt them.

With DocuTrack a user Alice who created an account with the dapp (using Internet Identity) can ask Bob to upload documents for her without having to create an account himself.

This feature makes it very easy and secure for service providers (e.g., a client advisor or wealth manager) to request documents of any type (e.g., house deeds, passport pictures or log files) from clients or other third parties.

Other document sharing apps require users to exchange public keys or other cryptographic material with which people typically struggle a lot.

## Table of content

- [How is DocuTrack implemented and how does the encryption/sharing work?](./architecture.md)
  - [Architecture](./architecture.md#Architecture)
  - [User Registration](./architecture.md#User-Registration)
  - [Document Upload](./architecture.md#Document-Upload)
  - [Document Access](./architecture.md#Document-Access)
  - [Cryptography](./architecture.md#Cryptography)
- [How can I use it?](./getting-started.md)
  - [Sign up](./getting-started.md#Sign-up)
  - [Uploading a file](./getting-started.md#Uploading-a-file)
  - [Sharing a file](./getting-started.md#Sharing-a-file)
  - [Revoking access to a file](./getting-started.md#Revoking-access-to-a-file)
- [Inter Canister Flows](./flows.md)
  - [Create a new user canister](./flows.md#Create-a-new-user-canister)
  - [Upload a document](./flows.md#Upload-a-document)
  - [Download a document](./flows.md#Download-a-document)
  - [Share a document](./flows.md#Share-a-document)
  - [Revoke access to a document](./flows.md#Revoke-access-to-a-document)
  - [Delete a document](./flows.md#Delete-a-document)
- [Canister API](./api.md)
  - [Orchestrator](./api.md#orchestrator-canister)
    - [create_user](./api.md#create_user)
    - [get_user](./api.md#get_user)
    - [get_users](./api.md#get_users)
    - [orbit_station](./api.md#orbit_station)
    - [retry_user_canister_creation](./api.md#retry_user_canister_creation)
    - [revoke_share_file](./api.md#revoke_share_file)
    - [revoke_share_file_for_users](./api.md#revoke_share_file_for_users)
    - [set_user](./api.md#set_user)
    - [share_file](./api.md#share_file)
    - [share_file_with_users](./api.md#share_file_with_users)
    - [shared_files](./api.md#shared_files)
    - [user_canister](./api.md#user_canister)
    - [username_exists](./api.md#username_exists)
    - [who_am_i](./api.md#who_am_i)
  - [User](./api.md#user-canister)
    - [delete_file](./api.md#delete_file)
    - [download_file](./api.md#download_file)
    - [get_alias_info](./api.md#get_alias_info)
    - [get_requests](./api.md#get_requests)
    - [get_shared_files](./api.md#get_shared_files)
    - [public_key](./api.md#public_key)
    - [request_file](./api.md#request_file)
    - [revoke_share](./api.md#revoke_share)
    - [set_public_key](./api.md#set_public_key)
    - [share_file](./api.md#share_file-2)
    - [share_file_with_users](./api.md#share_file_with_users-2)
    - [upload_file](./api.md#upload_file)
    - [upload_file_atomic](./api.md#upload_file_atomic)
    - [upload_file_continue](./api.md#upload_file_continue)
