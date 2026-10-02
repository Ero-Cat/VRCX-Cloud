/**
 * Local cache files are served by the server's image route; desktop
 * custom protocols no longer exist in this build.
 */
export function convertFileSrc(filePath: string, _protocol = 'asset'): string {
    const match = /([^/]+)\/([0-9]+)\.png$/.exec(filePath);
    if (match) {
        return `/api/img/${match[1]}/${match[2]}`;
    }
    return filePath;
}
