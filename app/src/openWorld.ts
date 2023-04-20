import { dataDir, homeDir } from '@tauri-apps/api/path';
import { type } from '@tauri-apps/api/os';

export async function savesDir(): Promise<string | null> {
    const osType = await type();

    switch (osType) {
        case 'Darwin':
            const data = await dataDir();
            return `${data}/minecraft/saves`;
        case 'Linux':
            const home = await homeDir();
            return `${home}/.minecraft/saves`;
        case 'Windows_NT':
            // TODO: Windows
            return null
    }
}
