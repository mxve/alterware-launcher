## todo
- [x] detect game
- [ ] downloading game
  - [x] better cdn setup
    - [x] hash based cdn urls to allow caching and still make sure the user alwys gets the correct file
      - `cdn_url/file_hash` `{ "name": "file/path/name.ext", "size": 123456, "hash": "file_hash" }`
    - [x] cdn fallback
    - [x] cdn url override
    - [x] ~~info.json -> stable.json, unstable.json~~, all files (named as hash) in ~~"files" dir~~ next to info.json and files in sub dir for stable/unstable
  - [ ] file download
    - [ ] http chunked transfer
    - [ ] resumable downloads
      - [ ] download to cache dir as hash name
      - [ ] detect & resume unfinished downloads
    - [ ] detect stuck downloads
  - [ ] file verification
    - [ ] blake3
    - [ ] file size

- [ ] UI????????????????????????????