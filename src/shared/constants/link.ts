import { vrchatPasswordUrl, vrchatRegisterUrl } from './vrchatWebUrls';

const links: Record<string, string> = {
    wiki: 'https://github.com/Ero-Cat/VRCX-Cloud/wiki',
    github: 'https://github.com/Ero-Cat/VRCX-Cloud',
    issues: 'https://github.com/Ero-Cat/VRCX-Cloud/issues',
    releases: 'https://github.com/Ero-Cat/VRCX-Cloud/releases',
    license: 'https://github.com/Ero-Cat/VRCX-Cloud/blob/master/LICENSE',
    vrchatStatus: 'https://status.vrchat.com/',
    vrchatDocsConfigurationFile:
        'https://docs.vrchat.com/docs/configuration-file',
    vrchatDocsLaunchOptions: 'https://docs.vrchat.com/docs/launch-options',
    vrchatPassword: vrchatPasswordUrl(),
    vrchatRegister: vrchatRegisterUrl()
};

export { links };
