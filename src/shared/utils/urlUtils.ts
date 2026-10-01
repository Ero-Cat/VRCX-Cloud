function getFaviconUrl(resource: unknown): string {
    if (!resource) {
        return '';
    }
    try {
        const url = new URL(String(resource));
        return `https://icons.duckduckgo.com/ip2/${url.host}.ico`;
    } catch (err) {
        console.error('Invalid URL:', resource, err);
        return '';
    }
}

function replaceVrcPackageUrl(url: unknown): string {
    if (!url) {
        return '';
    }
    return String(url).replace(
        'https://api.vrchat.cloud/',
        'https://vrchat.com/'
    );
}

export { getFaviconUrl, replaceVrcPackageUrl };
