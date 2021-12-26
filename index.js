
import './site.css';
import '@fortawesome/fontawesome-free/js/fontawesome'
import '@fortawesome/fontawesome-free/js/brands';
import tankGameImage from './images/tank.jpg';

import { greet, tank_game } from './pkg';
greet();

document.getElementById('tank-game-img').src = tankGameImage;

function hashHandler() {
  navigate();
}

window.addEventListener('hashchange', hashHandler, false);

function navigate() {
  if (location.hash === '#tank')
  {
    document.querySelector('.gallery').style.display = 'none';
    document.querySelector('#game').style.display = 'block';
    tank_game('canvas');

  } else {
    document.querySelector('.gallery').style.display = 'flex';
    document.querySelector('#game').style.display = 'none';
  }
}

navigate();
