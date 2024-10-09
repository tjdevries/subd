// Import necessary libraries
import * as THREE from 'three';
import Phaser from 'phaser';

// Initialize THREE.js scene
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add ambient light
const ambientLight = new THREE.AmbientLight(0x404040, 2);
scene.add(ambientLight);

// Add a rotating cube
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

camera.position.z = 5;

// Animate the scene
const animate = function () {
    requestAnimationFrame(animate);

    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;

    renderer.render(scene, camera);
};

animate();

// Initialize Phaser game
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
    this.load.image('neon-bg', 'assets/neon-background.png');
}

function create() {
    this.add.image(400, 300, 'neon-bg');

    // Add animated sprite representing digital water flow
    const water = this.add.rectangle(400, 300, 800, 600, 0x0000ff);
    water.alpha = 0.5;

    this.tweens.add({
        targets: water,
        alpha: 0.1,
        duration: 2000,
        ease: 'Power2',
        yoyo: true,
        repeat: -1
    });

    const text = this.add.text(400, 500, 'AI Top of the Pops', { fontSize: '32px', fill: '#fff' });
    text.setOrigin(0.5, 0.5);
}

function update() {
    // Example of constant animation
}

// Style adjustments (imaginary CSS influence)
const style = document.createElement('style');
style.innerHTML = `
    body {
        background-color: #1a1a1d;
        color: #cfcfcf;
        margin: 0;
        overflow: hidden;
        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    }
    .header-container, .nav-container {
        background-color: rgba(255, 255, 255, 0.1);
        padding: 20px;
        text-align: center;
    }
`;
document.head.appendChild(style);