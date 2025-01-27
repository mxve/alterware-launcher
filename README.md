## todo
- [x] detect game
- [ ] downloading game
  - [ ] better cdn setup
    - [ ] hash based cdn urls to allow caching and still make sure the user alwys gets the correct file
      - `cdn_url/file_hash` `{ "name": "file/path/name.ext", "size": 123456, "hash": "file_hash" }`
    - [ ] cdn fallback
    - [ ] cdn url override
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