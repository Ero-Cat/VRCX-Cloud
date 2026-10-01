function extractFileId(s: string): string {
    const match = s.match(/file_[0-9A-Za-z-]+/);
    return match ? match[0] : '';
}

function extractFileVersion(s: string): string {
    const match = /(?:\/file_[0-9A-Za-z-]+\/)([0-9]+)/gi.exec(s);
    return match ? match[1] : '';
}

function extractVariantVersion(url: string): string {
    if (!url) {
        return '0';
    }
    try {
        const params = new URLSearchParams(new URL(url).search);
        const version = params.get('v');
        if (version) {
            return version;
        }
        return '0';
    } catch {
        return '0';
    }
}

export { extractFileId, extractFileVersion, extractVariantVersion };
