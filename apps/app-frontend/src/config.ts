const trimTrailingSlash = (url: string) => url.replace(/\/$/, '')

export const AxolotlBrandConfig = Object.freeze({
    productName: 'StarryIsle Launcher',
    shortProductName: 'StarryIsle',
    website: 'https://github.com/yuxiaoao/starryisle-Launcher',
    repositoryUrl: 'https://github.com/yuxiaoao/starryisle-Launcher',
    supportUrl: 'https://github.com/yuxiaoao/starryisle-Launcher/issues',
    qqGroupNumber: '590809347',
    qqChannelUrl: 'https://pd.qq.com/s/c4xacb53e',
    sponsorUrl: '',
    bundleIdentifier: 'com.starryisle.launcher',
    deepLinkScheme: 'starryisle',
    userAgent: (version: string, os: string) => `StarryIsle-Launcher/${version} (${os})`,
    capabilities: Object.freeze({
        publicModrinthApi: true,
        ghsTelemetry: false,
    }),
})

const siteUrl = trimTrailingSlash(import.meta.env.MODRINTH_URL || 'https://modrinth.com')
const officialLabrinthBaseUrl = trimTrailingSlash(
    import.meta.env.MODRINTH_API_BASE_URL || 'https://api.modrinth.com',
)
type DownloadSourceMode = 'auto' | 'official_only' | 'mirror_preferred' | 'official_preferred'

// The Modrinth API always uses the official source; Modrinth download mirror
// routing is handled by the Rust download layer.
export function setModrinthSourceMode(_sourceMode: DownloadSourceMode) {}

export function setModrinthMirrorEnabled(_enabled: boolean) {}

export function getOfficialLabrinthBaseUrl() {
    return officialLabrinthBaseUrl
}

export function getLabrinthBaseUrl() {
    return officialLabrinthBaseUrl
}

export const config = {
    siteUrl,
    labrinthBaseUrl: getLabrinthBaseUrl,
}
