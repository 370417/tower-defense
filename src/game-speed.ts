import { bufferInput } from './input';

export let isSpedUp = false;

export function gameSpeed(): number {
    if (isSpedUp) {
        return 3;
    } else {
        return 1;
    }
}

const backButton = document.getElementsByClassName('back')[0] as HTMLButtonElement;
const playPauseButton = document.getElementsByClassName('pause')[0] as HTMLButtonElement;
const speedButton = document.getElementsByClassName('speed')[0] as HTMLButtonElement;
const nextButton = document.getElementsByClassName('next')[0] as HTMLButtonElement;

const waveDesc = document.querySelector('#wave-info span') as HTMLSpanElement;

playPauseButton.addEventListener('click', playPause);
speedButton.addEventListener('click', toggleSpeed);

function playPause() {
    bufferInput({ type: 'play pause' });
}

function toggleSpeed() {
    bufferInput({ type: 'fast forward' });
}

export function executeToggleSpeed(): void {
    isSpedUp = isSpedUp === false;
    if (isSpedUp) {
        speedButton.classList.add('fast');
    } else {
        speedButton.classList.remove('fast');
    }
}

backButton.addEventListener('click', () => {
    bufferInput({ type: 'skip back' });
});

nextButton.addEventListener('click', () => {
    bufferInput({ type: 'send next wave' });
});

// Because the game can decide to autopause itself, we need to check the
// play/pause/autopause state every frame and render on change.
let playPauseState = 1;
export function renderPlayPause(state: number): void {
    if (state !== playPauseState) {
        playPauseState = state;
        playPauseButton.dataset.state = ['paused', 'autopaused', 'playing'][state];
    }
}

window.addEventListener('keydown', event => {
    if (!event.repeat) {
        switch (event.code) {
            case 'KeyH':
                bufferInput({ type: 'skip back' });
                event.preventDefault();
                break;
            case 'KeyJ':
            case 'Space':
                bufferInput({ type: 'play pause' });
                event.preventDefault();
                break;
            case 'KeyK':
                bufferInput({ type: 'fast forward' });
                event.preventDefault();
                break;
            case 'KeyL':
                bufferInput({ type: 'send next wave' });
                event.preventDefault();
                break;
        }
    }
});

export function renderWaveDesc(waveIndex: number, ticksTillWave: number): void {
    let text = '';
    if (waveIndex < 0) {
        text = '';
    } else if (ticksTillWave > 0) {
        const seconds = Math.ceil(ticksTillWave / 60);
        text = `Wave ${waveIndex + 1} arrives in ${seconds}s`;
    } else {
        text = `Sending wave ${waveIndex + 1}`;
    }
    if (text !== waveDesc.textContent) {
        waveDesc.textContent = text;
    }
}
