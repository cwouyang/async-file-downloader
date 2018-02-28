# async-file-downloader
Given an URL which responses a JSON array containing a list of target URLs. This program downloads files from the target URLs and satisfy following conditions.
1. The binary accepts an input of the API URL from arguments. For example,
    ```shell
    > binary JSON_URL
    ```
2. The binary must output total download progress every second during downloading on the terminal in percentage regarding number of downloaded bytes.
3. Files must be downloaded concurrently with at least two threads.
4. The binary must output MD5 hash of each file when it is downloaded.
