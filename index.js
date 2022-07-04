
import './site.css';

import { start_game } from './pkg';

start_game('canvas');

document.getElementById('play').onclick = function() {
    document.getElementById('instructions').remove();
    document.getElementById('background-music').play();
}

window.addEventListener('message', function (message) {
    //console.log('Received message from game: ', message.data);

    let state = message.data;

    if (state.current_player != null) {
        document.getElementById('player').innerText = (state.current_player + 1) + "";
    }

    if (state.player_color) {
        document.getElementById('turn').style = `color: ${state.player_color}`;
    }

    if (state.cannon_power) {
        document.getElementById('power').innerText = `${state.cannon_power}`;
    }

    if (state.game_over) {
        document.getElementById('winner').style = `color: ${state.player_color}`;
        document.getElementById('winner-id').innerText = (state.current_player + 1) + "";
        document.getElementById('game-over').style = '';
    }
});
