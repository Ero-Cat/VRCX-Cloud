/**
 * Small JSON file KV the desktop stored in its app cache; on the web the
 * same content lives in localStorage.
 */
export async function readLocalTextFile(fileName: string): Promise<string> {
    const value = window.localStorage.getItem(`vrcx-file:${fileName}`);
    if (value === null) {
        throw new Error(`file not found: ${fileName}`);
    }
    return value;
}

export async function writeLocalTextFile(
    fileName: string,
    contents: string
): Promise<void> {
    window.localStorage.setItem(`vrcx-file:${fileName}`, contents);
}
