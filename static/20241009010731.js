// Import necessary libraries
import * as THREE from 'three';
import Phaser from 'phaser';

// Scene initialization
function initScene() {
  // Initialize Three.js Scene
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
  const renderer = new THREE.WebGLRenderer();
  renderer.setSize(window.innerWidth, window.innerHeight);
  document.body.appendChild(renderer.domElement);

  // Add lights
  const light = new THREE.AmbientLight(0x404040); 
  scene.add(light);

  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.5);
  scene.add(directionalLight);

  // Create a moon-like object
  const geometry = new THREE.SphereGeometry(1, 32, 32);
  const material = new THREE.MeshStandardMaterial({ color: 0xffffcc });
  const sphere = new THREE.Mesh(geometry, material);
  scene.add(sphere);

  // Moon Animation
  function animateMoon() {
    requestAnimationFrame(animateMoon);
    sphere.rotation.x += 0.01;
    sphere.rotation.y += 0.01;
    renderer.render(scene, camera);
  }
  animateMoon();

  camera.position.z = 5;

  // Initialize Phaser Game
  const config = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    scene: {
      preload: preload,
      create: create,
      update: update
    }
  };

  const game = new Phaser.Game(config);

  function preload() {
    this.load.image('moon', '/static/moon-image.png');
  }

  function create() {
    // Create a fun moonlit background
    this.add.image(400, 300, 'moon').setDepth(0);

    let bouncyBall = this.physics.add.image(400, 300, 'moon').setDepth(1);
    bouncyBall.setVelocity(200, 200);
    bouncyBall.setBounce(1, 1);
    bouncyBall.setCollideWorldBounds(true);
  }

  function update() {
    // Update logic
  }
}

// Run the scene initialization
initScene();

// Styling for the HTML
const styles = `
  body {
    background: linear-gradient(to right, #000428, #004e92);
    color: white;
    font-family: 'Arial', sans-serif;
    overflow: hidden;
  }

  .header-container, .nav-container, .unplayed_songs, .current-song, .users-container, .charts-container {
    margin: 20px auto;
    text-align: center;
    background-color: rgba(0, 0, 0, 0.5);
    padding: 20px;
    border-radius: 10px;
    box-shadow: 0 0 15px rgba(0, 0, 0, 0.2);
  }

  h1, h2, h3, h4 {
    color: #ffe600;
    text-shadow: 1px 1px 2px black;
  }

  a {
    color: #00d1b2;
    text-decoration: none;
    transition: color 0.3s ease;
  }

  a:hover {
    color: #ffdd57;
  }

  .nav-link {
    margin: 15px;
    transition: transform 0.3s ease;
  }

  .nav-link:hover {
    transform: scale(1.1);
  }

  .song, .user, .chart-container, .unplayed_song {
    transition: transform 0.3s ease;
  }

  .song:hover, .user:hover, .chart-container:hover, .unplayed_song:hover {
    transform: scale(1.05);
  }

  .videos video {
    border: 2px solid #fff;
    border-radius: 5px;
    margin: 10px;
  }
`;

// Append the styles to the page
const styleSheet = document.createElement('style');
styleSheet.type = 'text/css';
styleSheet.innerText = styles;
document.head.appendChild(styleSheet);