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

playPauseButton.addEventListener('click', playPause);
speedButton.addEventListener('click', toggleSpeed);

function playPause() {
    bufferInput({ type: 'play pause' });
    // if (isPaused) {
    //     isPaused = false;
    //     playPauseButton.classList.replace('play', 'pause');
    // } else {
    //     isPaused = true;
    //     playPauseButton.classList.replace('pause', 'play');
    // }
}

function toggleSpeed() {
    bufferInput({ type: 'fast forward' });
    // if (isSpedUp) {
    //     isSpedUp = false;
    //     speedButton.classList.remove('fast');
    // } else {
    //     isSpedUp = true;
    //     speedButton.classList.add('fast');
    // }
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
