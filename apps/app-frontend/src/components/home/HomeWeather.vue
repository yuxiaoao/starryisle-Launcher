<script setup lang="ts">
import { RefreshCwIcon, SearchIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import { CHINA_CITIES } from '@/components/home/china-cities'
import {
    HOME_WEATHER_DEFAULT_CITY,
    HOME_WEATHER_REFRESH_MS,
    type HomeWeatherCity,
    type HomeWidgetSize,
} from '@/components/home/home-dashboard'

const props = defineProps<{
    city?: HomeWeatherCity | null
    dashboardSize?: HomeWidgetSize | null
}>()

const emit = defineEmits<{
    'change-city': [city: HomeWeatherCity]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
    title: {
        id: 'app.home.weather.title',
        defaultMessage: 'Weather',
    },
    changeCity: {
        id: 'app.home.weather.change-city',
        defaultMessage: 'Change city',
    },
    searchPlaceholder: {
        id: 'app.home.weather.search-placeholder',
        defaultMessage: 'Search for a city',
    },
    searching: {
        id: 'app.home.weather.searching',
        defaultMessage: 'Searching…',
    },
    noResults: {
        id: 'app.home.weather.no-results',
        defaultMessage: 'No matching city',
    },
    loading: {
        id: 'app.home.weather.loading',
        defaultMessage: 'Loading weather…',
    },
    loadFailed: {
        id: 'app.home.weather.load-failed',
        defaultMessage: 'Weather is unavailable. Try again later.',
    },
    refresh: {
        id: 'app.home.weather.refresh',
        defaultMessage: 'Refresh',
    },
    feelsLike: {
        id: 'app.home.weather.feels-like',
        defaultMessage: 'Feels like',
    },
    high: {
        id: 'app.home.weather.high',
        defaultMessage: 'High',
    },
    low: {
        id: 'app.home.weather.low',
        defaultMessage: 'Low',
    },
    humidity: {
        id: 'app.home.weather.humidity',
        defaultMessage: 'Humidity',
    },
    wind: {
        id: 'app.home.weather.wind',
        defaultMessage: 'Wind',
    },
    updatedAt: {
        id: 'app.home.weather.updated-at',
        defaultMessage: 'Updated {time}',
    },
    c0: {
        id: 'app.home.weather.condition.sunny',
        defaultMessage: 'Clear',
    },
    c1: {
        id: 'app.home.weather.condition.mostly-clear',
        defaultMessage: 'Mostly clear',
    },
    c2: {
        id: 'app.home.weather.condition.partly-cloudy',
        defaultMessage: 'Partly cloudy',
    },
    c3: {
        id: 'app.home.weather.condition.overcast',
        defaultMessage: 'Overcast',
    },
    c45: {
        id: 'app.home.weather.condition.fog',
        defaultMessage: 'Fog',
    },
    c48: {
        id: 'app.home.weather.condition.rime-fog',
        defaultMessage: 'Depositing rime fog',
    },
    c51: {
        id: 'app.home.weather.condition.light-drizzle',
        defaultMessage: 'Light drizzle',
    },
    c53: {
        id: 'app.home.weather.condition.drizzle',
        defaultMessage: 'Drizzle',
    },
    c55: {
        id: 'app.home.weather.condition.dense-drizzle',
        defaultMessage: 'Dense drizzle',
    },
    c56: {
        id: 'app.home.weather.condition.light-freezing-drizzle',
        defaultMessage: 'Light freezing drizzle',
    },
    c57: {
        id: 'app.home.weather.condition.freezing-drizzle',
        defaultMessage: 'Freezing drizzle',
    },
    c61: {
        id: 'app.home.weather.condition.light-rain',
        defaultMessage: 'Light rain',
    },
    c63: {
        id: 'app.home.weather.condition.rain',
        defaultMessage: 'Rain',
    },
    c65: {
        id: 'app.home.weather.condition.heavy-rain',
        defaultMessage: 'Heavy rain',
    },
    c66: {
        id: 'app.home.weather.condition.light-freezing-rain',
        defaultMessage: 'Light freezing rain',
    },
    c67: {
        id: 'app.home.weather.condition.freezing-rain',
        defaultMessage: 'Freezing rain',
    },
    c71: {
        id: 'app.home.weather.condition.light-snow',
        defaultMessage: 'Light snow',
    },
    c73: {
        id: 'app.home.weather.condition.snow',
        defaultMessage: 'Snow',
    },
    c75: {
        id: 'app.home.weather.condition.heavy-snow',
        defaultMessage: 'Heavy snow',
    },
    c77: {
        id: 'app.home.weather.condition.snow-grains',
        defaultMessage: 'Snow grains',
    },
    c80: {
        id: 'app.home.weather.condition.light-showers',
        defaultMessage: 'Light showers',
    },
    c81: {
        id: 'app.home.weather.condition.showers',
        defaultMessage: 'Showers',
    },
    c82: {
        id: 'app.home.weather.condition.violent-showers',
        defaultMessage: 'Violent showers',
    },
    c85: {
        id: 'app.home.weather.condition.light-snow-showers',
        defaultMessage: 'Light snow showers',
    },
    c86: {
        id: 'app.home.weather.condition.snow-showers',
        defaultMessage: 'Snow showers',
    },
    c95: {
        id: 'app.home.weather.condition.thunderstorm',
        defaultMessage: 'Thunderstorm',
    },
    c96: {
        id: 'app.home.weather.condition.thunderstorm-hail',
        defaultMessage: 'Thunderstorm with hail',
    },
    c99: {
        id: 'app.home.weather.condition.severe-thunderstorm-hail',
        defaultMessage: 'Severe thunderstorm with hail',
    },
})

/** WMO weather codes, grouped so one label covers the neighbouring codes. */
const WEATHER_CODES: Record<number, { message: keyof typeof messages; icon: string }> = {
    0: { message: 'c0', icon: '☀️' },
    1: { message: 'c1', icon: '🌤️' },
    2: { message: 'c2', icon: '⛅' },
    3: { message: 'c3', icon: '☁️' },
    45: { message: 'c45', icon: '🌫️' },
    48: { message: 'c48', icon: '🌫️' },
    51: { message: 'c51', icon: '🌦️' },
    53: { message: 'c53', icon: '🌦️' },
    55: { message: 'c55', icon: '🌦️' },
    56: { message: 'c56', icon: '🌧️' },
    57: { message: 'c57', icon: '🌧️' },
    61: { message: 'c61', icon: '🌧️' },
    63: { message: 'c63', icon: '🌧️' },
    65: { message: 'c65', icon: '🌧️' },
    66: { message: 'c66', icon: '🌧️' },
    67: { message: 'c67', icon: '🌧️' },
    71: { message: 'c71', icon: '🌨️' },
    73: { message: 'c73', icon: '🌨️' },
    75: { message: 'c75', icon: '❄️' },
    77: { message: 'c77', icon: '🌨️' },
    80: { message: 'c80', icon: '🌦️' },
    81: { message: 'c81', icon: '🌧️' },
    82: { message: 'c82', icon: '⛈️' },
    85: { message: 'c85', icon: '🌨️' },
    86: { message: 'c86', icon: '🌨️' },
    95: { message: 'c95', icon: '⛈️' },
    96: { message: 'c96', icon: '⛈️' },
    99: { message: 'c99', icon: '⛈️' },
}

type WeatherData = {
    temperature: number
    apparentTemperature: number
    humidity: number
    windSpeed: number
    weatherCode: number
    high: number
    low: number
}

const activeCity = computed(() => props.city ?? HOME_WEATHER_DEFAULT_CITY)
const weather = ref<WeatherData | null>(null)
const loading = ref(false)
const failed = ref(false)
const updatedAt = ref<Date | null>(null)

const cityPanelOpen = ref(false)
const searchQuery = ref('')
const searchResults = ref<HomeWeatherCity[]>([])
const searching = ref(false)

let refreshTimer: ReturnType<typeof setInterval> | null = null
let searchTimer: ReturnType<typeof setTimeout> | null = null
let searchGeneration = 0
let weatherGeneration = 0

const WEATHER_CACHE_PREFIX = 'home-weather:v1:'

/// Last successful reading, so an offline or rate-limited refresh can still show
/// today's weather instead of an error.
function readWeatherCache(city: HomeWeatherCity): {
    weather: WeatherData
    updatedAt: Date
} | null {
    try {
        const raw = window.localStorage.getItem(`${WEATHER_CACHE_PREFIX}${city.id}`)
        if (!raw) return null
        const parsed = JSON.parse(raw) as { weather: WeatherData; updatedAt: string }
        if (!parsed?.weather || !parsed.updatedAt) return null
        const updatedAt = new Date(parsed.updatedAt)
        if (Number.isNaN(updatedAt.getTime())) return null
        // Older than a day is not worth showing as current weather.
        if (Date.now() - updatedAt.getTime() > 24 * 60 * 60 * 1000) return null
        return { weather: parsed.weather, updatedAt }
    } catch {
        return null
    }
}

function writeWeatherCache(city: HomeWeatherCity, data: WeatherData, at: Date) {
    try {
        window.localStorage.setItem(
            `${WEATHER_CACHE_PREFIX}${city.id}`,
            JSON.stringify({ weather: data, updatedAt: at.toISOString() }),
        )
    } catch {
        // Storage may be unavailable; caching is only a fallback.
    }
}

const condition = computed(() => WEATHER_CODES[weather.value?.weatherCode ?? 0] ?? WEATHER_CODES[3])
const conditionLabel = computed(() => formatMessage(messages[condition.value.message]))
const updatedLabel = computed(() => {
    if (!updatedAt.value) return ''
    const time = updatedAt.value.toLocaleTimeString(undefined, {
        hour: '2-digit',
        minute: '2-digit',
    })
    return formatMessage(messages.updatedAt, { time })
})

async function loadWeather() {
    const { latitude, longitude } = activeCity.value
    const generation = ++weatherGeneration
    loading.value = true
    failed.value = false
    try {
        const url = new URL('https://api.open-meteo.com/v1/forecast')
        url.searchParams.set('latitude', String(latitude))
        url.searchParams.set('longitude', String(longitude))
        url.searchParams.set(
            'current',
            'temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m',
        )
        url.searchParams.set('daily', 'temperature_2m_max,temperature_2m_min')
        url.searchParams.set('forecast_days', '1')
        url.searchParams.set('timezone', 'auto')

        const response = await fetch(url, { signal: AbortSignal.timeout(10000) })
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        const payload = await response.json()
        if (generation !== weatherGeneration) return

        weather.value = {
            temperature: Math.round(payload.current.temperature_2m),
            apparentTemperature: Math.round(payload.current.apparent_temperature),
            humidity: Math.round(payload.current.relative_humidity_2m),
            windSpeed: Math.round(payload.current.wind_speed_10m),
            weatherCode: Number(payload.current.weather_code),
            high: Math.round(payload.daily.temperature_2m_max[0]),
            low: Math.round(payload.daily.temperature_2m_min[0]),
        }
        updatedAt.value = new Date()
        writeWeatherCache(activeCity.value, weather.value, updatedAt.value)
    } catch {
        if (generation !== weatherGeneration) return
        // Keep the last successful reading so the widget still shows something offline.
        const cached = readWeatherCache(activeCity.value)
        if (cached) {
            weather.value = cached.weather
            updatedAt.value = cached.updatedAt
        } else {
            failed.value = true
        }
    } finally {
        if (generation === weatherGeneration) loading.value = false
    }
}

/** Chinese names for well-known foreign cities: searchable name plus the country they belong to. */
const FOREIGN_CITY_ALIASES: Record<string, { name: string; country: string }> = {
    东京: { name: 'Tokyo', country: '日本' },
    大阪: { name: 'Osaka', country: '日本' },
    京都: { name: 'Kyoto', country: '日本' },
    名古屋: { name: 'Nagoya', country: '日本' },
    札幌: { name: 'Sapporo', country: '日本' },
    福冈: { name: 'Fukuoka', country: '日本' },
    首尔: { name: 'Seoul', country: '韩国' },
    釜山: { name: 'Busan', country: '韩国' },
    平壤: { name: 'Pyongyang', country: '朝鲜' },
    新加坡: { name: 'Singapore', country: '新加坡' },
    曼谷: { name: 'Bangkok', country: '泰国' },
    吉隆坡: { name: 'Kuala Lumpur', country: '马来西亚' },
    雅加达: { name: 'Jakarta', country: '印度尼西亚' },
    马尼拉: { name: 'Manila', country: '菲律宾' },
    河内: { name: 'Hanoi', country: '越南' },
    胡志明市: { name: 'Ho Chi Minh City', country: '越南' },
    新德里: { name: 'New Delhi', country: '印度' },
    孟买: { name: 'Mumbai', country: '印度' },
    迪拜: { name: 'Dubai', country: '阿联酋' },
    伊斯坦布尔: { name: 'Istanbul', country: '土耳其' },
    莫斯科: { name: 'Moscow', country: '俄罗斯' },
    圣彼得堡: { name: 'Saint Petersburg', country: '俄罗斯' },
    伦敦: { name: 'London', country: '英国' },
    巴黎: { name: 'Paris', country: '法国' },
    柏林: { name: 'Berlin', country: '德国' },
    慕尼黑: { name: 'Munich', country: '德国' },
    罗马: { name: 'Rome', country: '意大利' },
    米兰: { name: 'Milan', country: '意大利' },
    马德里: { name: 'Madrid', country: '西班牙' },
    巴塞罗那: { name: 'Barcelona', country: '西班牙' },
    阿姆斯特丹: { name: 'Amsterdam', country: '荷兰' },
    布鲁塞尔: { name: 'Brussels', country: '比利时' },
    维也纳: { name: 'Vienna', country: '奥地利' },
    苏黎世: { name: 'Zurich', country: '瑞士' },
    斯德哥尔摩: { name: 'Stockholm', country: '瑞典' },
    哥本哈根: { name: 'Copenhagen', country: '丹麦' },
    赫尔辛基: { name: 'Helsinki', country: '芬兰' },
    华沙: { name: 'Warsaw', country: '波兰' },
    布拉格: { name: 'Prague', country: '捷克' },
    里斯本: { name: 'Lisbon', country: '葡萄牙' },
    雅典: { name: 'Athens', country: '希腊' },
    纽约: { name: 'New York', country: '美国' },
    洛杉矶: { name: 'Los Angeles', country: '美国' },
    旧金山: { name: 'San Francisco', country: '美国' },
    西雅图: { name: 'Seattle', country: '美国' },
    芝加哥: { name: 'Chicago', country: '美国' },
    波士顿: { name: 'Boston', country: '美国' },
    华盛顿: { name: 'Washington', country: '美国' },
    温哥华: { name: 'Vancouver', country: '加拿大' },
    多伦多: { name: 'Toronto', country: '加拿大' },
    蒙特利尔: { name: 'Montreal', country: '加拿大' },
    墨西哥城: { name: 'Mexico City', country: '墨西哥' },
    圣保罗: { name: 'Sao Paulo', country: '巴西' },
    里约热内卢: { name: 'Rio de Janeiro', country: '巴西' },
    布宜诺斯艾利斯: { name: 'Buenos Aires', country: '阿根廷' },
    悉尼: { name: 'Sydney', country: '澳大利亚' },
    墨尔本: { name: 'Melbourne', country: '澳大利亚' },
    奥克兰: { name: 'Auckland', country: '新西兰' },
    开罗: { name: 'Cairo', country: '埃及' },
    约翰内斯堡: { name: 'Johannesburg', country: '南非' },
    内罗毕: { name: 'Nairobi', country: '肯尼亚' },
}

/** Province and autonomous-region names resolve to their capital city. */
const PROVINCE_CAPITALS: Record<string, string> = {
    河北: '石家庄',
    山西: '太原',
    内蒙古: '呼和浩特',
    辽宁: '沈阳',
    吉林: '长春',
    黑龙江: '哈尔滨',
    江苏: '南京',
    浙江: '杭州',
    安徽: '合肥',
    福建: '福州',
    江西: '南昌',
    山东: '济南',
    河南: '郑州',
    湖北: '武汉',
    湖南: '长沙',
    广东: '广州',
    广西: '南宁',
    海南: '海口',
    四川: '成都',
    贵州: '贵阳',
    云南: '昆明',
    西藏: '拉萨',
    陕西: '西安',
    甘肃: '兰州',
    青海: '西宁',
    宁夏: '银川',
    新疆: '乌鲁木齐',
    台湾: '台北',
    香港: '香港',
    澳门: '澳门',
}

/** Regions whose country label has to carry 中国 in front. */
const CHINA_REGION_PREFIX = new Set(['香港', '澳门', '台湾'])

/** Villages share names with real cities, so weak matches are dropped. */
const MIN_CITY_POPULATION = 100_000

function normalizeCountry(country: string | undefined): string | undefined {
    if (!country) return country
    return CHINA_REGION_PREFIX.has(country) ? `中国${country}` : country
}

function scheduleSearch() {
    if (searchTimer) clearTimeout(searchTimer)
    const query = searchQuery.value.trim()
    if (query.length < 1) {
        searchResults.value = []
        searching.value = false
        return
    }
    searching.value = true
    searchTimer = setTimeout(async () => {
        const generation = ++searchGeneration
        try {
            // Chinese cities resolve locally: the online geocoder mixes villages into
            // results and frequently reports the wrong province.
            const local = searchChinaCities(query)
            if (local.length > 0) {
                if (generation === searchGeneration) searchResults.value = local
                return
            }

            const bareQuery = query.replace(/(省|市|自治区|特别行政区)$/, '')
            const capital = PROVINCE_CAPITALS[bareQuery]
            // GeoNames (the index behind this API) matches Latin names, so a Chinese
            // name for a foreign city has to be translated before searching.
            const alias = FOREIGN_CITY_ALIASES[query]
            let results = await fetchCities(capital ?? alias?.name ?? query, alias?.country)
            if (results.length === 0 && alias) results = await fetchCities(query)
            if (results.length === 0 && capital) results = await fetchCities(query)
            // A slower earlier query must not overwrite the newer results.
            if (generation !== searchGeneration) return
            searchResults.value = results
        } catch {
            if (generation !== searchGeneration) return
            searchResults.value = []
        } finally {
            if (generation === searchGeneration) searching.value = false
        }
    }, 350)
}

/** Local lookup: exact city name first, then a prefix match. */
function searchChinaCities(rawQuery: string): HomeWeatherCity[] {
    const bare = rawQuery.replace(/(省|市|自治区|特别行政区)$/, '')
    const target = PROVINCE_CAPITALS[bare] ?? bare
    const exact = CHINA_CITIES.filter((city) => city.name === target)
    const matches = exact.length > 0 ? exact : CHINA_CITIES.filter((city) => city.name.startsWith(target))
    return matches.slice(0, 5).map((city) => ({
        id: `cn:${city.name}`,
        name: city.name,
        region: city.province.startsWith('中国')
            ? city.province
            : city.province === city.name
              ? '中国'
              : `${city.province} · 中国`,
        latitude: city.latitude,
        longitude: city.longitude,
    }))
}

type GeoResult = {
    id: number
    name: string
    admin1?: string
    country?: string
    population?: number
    latitude: number
    longitude: number
}

async function fetchCities(name: string, country?: string): Promise<HomeWeatherCity[]> {
    const url = new URL('https://geocoding-api.open-meteo.com/v1/search')
    url.searchParams.set('name', name)
    url.searchParams.set('count', '30')
    url.searchParams.set('language', 'zh')
    url.searchParams.set('format', 'json')
    const response = await fetch(url, { signal: AbortSignal.timeout(8000) })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const payload = await response.json()

    const deduped = new Map<number, GeoResult>()
    for (const result of (payload.results ?? []) as GeoResult[]) {
        if (!deduped.has(result.id)) deduped.set(result.id, result)
    }
    const results = [...deduped.values()].sort(
        (a, b) => (b.population ?? 0) - (a.population ?? 0),
    )

    // A translated Chinese name is ambiguous ("巴黎" is also a village in Guangxi and
    // two towns in the US), so keep only the country the alias points at.
    const scoped = country ? results.filter((result) => result.country === country) : results
    const major = scoped.filter((result) => (result.population ?? 0) >= MIN_CITY_POPULATION)
    const chosen = major.length > 0 ? major : scoped.length > 0 ? scoped : results

    return chosen.slice(0, 5).map((result) => ({
        id: String(result.id),
        name: result.name,
        // Repeating the city name as the region ("Tokyo · Tokyo · Japan") reads badly.
        region: [
            result.admin1 && result.admin1 !== result.name ? result.admin1 : null,
            normalizeCountry(result.country),
        ]
            .filter(Boolean)
            .join(' · '),
        latitude: result.latitude,
        longitude: result.longitude,
    }))
}

function selectCity(city: HomeWeatherCity) {
    emit('change-city', city)
    closeCityPanel()
}

const cityButtonEl = ref<HTMLButtonElement | null>(null)
const cityPanelEl = ref<HTMLElement | null>(null)
const searchInputEl = ref<HTMLInputElement | null>(null)
const cityPanelStyle = ref<Record<string, string>>({})

function closeCityPanel() {
    cityPanelOpen.value = false
    searchQuery.value = ''
    searchResults.value = []
}

async function toggleCityPanel() {
    if (cityPanelOpen.value) {
        closeCityPanel()
        return
    }
    cityPanelOpen.value = true
    await nextTick()
    // The panel is teleported to <body>: the widget card clips overflow, so an
    // in-flow panel would be cut off as soon as it is taller than the card.
    const anchor = cityButtonEl.value
    if (!anchor) return
    const rect = anchor.getBoundingClientRect()
    const width = Math.max(220, Math.min(320, window.innerWidth - 16))
    cityPanelStyle.value = {
        position: 'fixed',
        top: `${rect.bottom + 6}px`,
        left: `${Math.min(Math.max(8, rect.left), window.innerWidth - width - 8)}px`,
        width: `${width}px`,
    }
    searchInputEl.value?.focus()
}

function handleDocumentPointerDown(event: PointerEvent) {
    if (!cityPanelOpen.value) return
    const target = event.target as Node | null
    if (!target) return
    if (cityPanelEl.value?.contains(target) || cityButtonEl.value?.contains(target)) return
    closeCityPanel()
}

watch(searchQuery, scheduleSearch)
watch(
    () => activeCity.value.id,
    () => loadWeather(),
)

onMounted(() => {
    loadWeather()
    refreshTimer = setInterval(() => {
            // No point polling while the window is in the background.
            if (!document.hidden) void loadWeather()
        }, HOME_WEATHER_REFRESH_MS)
    document.addEventListener('pointerdown', handleDocumentPointerDown, true)
})

onUnmounted(() => {
    if (refreshTimer) clearInterval(refreshTimer)
    if (searchTimer) clearTimeout(searchTimer)
    document.removeEventListener('pointerdown', handleDocumentPointerDown, true)
})
</script>

<template>
    <section class="home-weather flex h-full min-h-0 min-w-0 flex-col gap-3">
        <div class="flex h-8 min-w-0 flex-none items-center gap-2">
            <h2 class="home-weather-heading">{{ formatMessage(messages.title) }}</h2>
            <button
                ref="cityButtonEl"
                type="button"
                class="home-weather-city"
                v-tooltip="formatMessage(messages.changeCity)"
                @click="toggleCityPanel"
            >
                {{ activeCity.name }}
            </button>
            <button
                type="button"
                class="home-weather-refresh ml-auto"
                v-tooltip="formatMessage(messages.refresh)"
                @click="loadWeather"
            >
                <RefreshCwIcon :class="{ 'animate-spin': loading }" />
            </button>
        </div>

        <Teleport to="body">
            <div
                v-if="cityPanelOpen"
                ref="cityPanelEl"
                class="home-weather-city-panel"
                :style="cityPanelStyle"
            >
                <div class="home-weather-search">
                    <SearchIcon />
                    <input
                        ref="searchInputEl"
                        v-model="searchQuery"
                        type="text"
                        :placeholder="formatMessage(messages.searchPlaceholder)"
                    />
                </div>
                <p v-if="searching" class="home-weather-hint">
                    {{ formatMessage(messages.searching) }}
                </p>
                <p v-else-if="searchQuery && searchResults.length === 0" class="home-weather-hint">
                    {{ formatMessage(messages.noResults) }}
                </p>
                <button
                    v-for="result in searchResults"
                    :key="result.id"
                    type="button"
                    class="home-weather-result"
                    @click="selectCity(result)"
                >
                    <span>{{ result.name }}</span>
                    <span v-if="result.region" class="home-weather-region">{{ result.region }}</span>
                </button>
            </div>
        </Teleport>

        <div v-if="failed" class="home-weather-state">
            <span>{{ formatMessage(messages.loadFailed) }}</span>
        </div>
        <div v-else-if="!weather" class="home-weather-state">
            <span>{{ formatMessage(messages.loading) }}</span>
        </div>
        <div v-else class="home-weather-body">
            <div class="home-weather-main">
                <span class="home-weather-icon" aria-hidden="true">{{ condition.icon }}</span>
                <span class="home-weather-temp">{{ weather.temperature }}°</span>
                <div class="home-weather-meta">
                    <span class="home-weather-condition">{{ conditionLabel }}</span>
                    <span class="home-weather-sub">
                        {{ formatMessage(messages.feelsLike) }} {{ weather.apparentTemperature }}°
                    </span>
                </div>
            </div>
            <div class="home-weather-stats">
                <span>
                    {{ formatMessage(messages.high) }} {{ weather.high }}° ·
                    {{ formatMessage(messages.low) }} {{ weather.low }}°
                </span>
                <span>
                    {{ formatMessage(messages.humidity) }} {{ weather.humidity }}% ·
                    {{ formatMessage(messages.wind) }} {{ weather.windSpeed }} km/h
                </span>
            </div>
            <span v-if="updatedLabel" class="home-weather-updated">{{ updatedLabel }}</span>
        </div>
    </section>
</template>

<style scoped>
.home-weather-heading {
    min-width: 0;
    overflow: hidden;
    margin: 0;
    color: var(--color-text-primary);
    font-size: 1rem;
    font-weight: 700;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.home-weather-city {
    overflow: hidden;
    max-width: 8rem;
    padding: 0;
    border: none;
    background: none;
    color: var(--color-text-tertiary);
    font-size: 0.8125rem;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: pointer;
}

.home-weather-city:hover {
    color: var(--color-text-primary);
}

.home-weather-refresh {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    padding: 0;
    border: none;
    border-radius: 999px;
    background: transparent;
    color: var(--color-text-tertiary);
    cursor: pointer;
}

.home-weather-refresh:hover {
    background: var(--surface-4);
    color: var(--color-text-primary);
}

.home-weather-refresh :deep(svg) {
    width: 1rem;
    height: 1rem;
}

.home-weather-city-panel {
    z-index: 60;
    display: flex;
    max-height: 20rem;
    flex-direction: column;
    gap: 0.25rem;
    overflow-y: auto;
    padding: 0.5rem;
    border: 1px solid var(--color-divider);
    border-radius: var(--radius-lg);
    background: var(--surface-2);
    box-shadow: 0 12px 28px rgb(0 0 0 / 24%);
}

.home-weather-search {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-md);
    background: var(--surface-3);
}

.home-weather-search :deep(svg) {
    width: 0.875rem;
    height: 0.875rem;
    color: var(--color-text-tertiary);
}

.home-weather-search input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    font-size: 0.875rem;
    outline: none;
}

.home-weather-search input:focus,
.home-weather-search input:focus-visible {
    outline: none;
    box-shadow: none;
}

.home-weather-hint {
    margin: 0;
    padding: 0.25rem 0.5rem;
    color: var(--color-text-tertiary);
    font-size: 0.8125rem;
}

.home-weather-result {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--color-text-primary);
    font-size: 0.875rem;
    text-align: left;
    cursor: pointer;
}

.home-weather-result:hover {
    background: var(--surface-4);
}

.home-weather-region {
    color: var(--color-text-tertiary);
    font-size: 0.75rem;
}

.home-weather-state {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary);
    font-size: 0.875rem;
}

.home-weather-body {
    display: flex;
    flex: 1;
    min-height: 0;
    flex-direction: column;
    align-items: flex-start;
    justify-content: flex-start;
    gap: 0.5rem;
    overflow: hidden;
}

.home-weather-main {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 0.75rem;
    min-width: 0;
}

.home-weather-icon {
    font-size: 2rem;
    line-height: 1;
}

.home-weather-temp {
    color: var(--color-text-primary);
    font-size: 2rem;
    font-weight: 700;
    line-height: 1;
}

.home-weather-meta {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 0.125rem;
}

.home-weather-condition {
    overflow: hidden;
    color: var(--color-text-primary);
    font-size: 0.9375rem;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.home-weather-sub {
    color: var(--color-text-tertiary);
    font-size: 0.8125rem;
}

.home-weather-stats {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem 1rem;
    color: var(--color-text-tertiary);
    font-size: 0.8125rem;
}

.home-weather-updated {
    margin-top: auto;
    color: var(--color-text-tertiary);
    font-size: 0.75rem;
}
</style>
