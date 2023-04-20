import { dataDir, homeDir } from '@tauri-apps/api/path';
import { type } from 'os';

export async function savesDir(): Promise<string | null> {
    const osType = await type();

    switch (osType) {
        // TODO: Windows
        case 'Darwin':
            const data = await dataDir();
            return `${data}/minecraft/saves`;
        case 'Linux':
            const home = await homeDir();
            return `${home}/.minecraft/saves`;
        default:
            return null;
    }
}
