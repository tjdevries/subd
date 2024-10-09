// Import necessary libraries for 3D and animations
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Phaser from 'phaser';

// Create a 3D environment using Three.js
function initThreeJsScene() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer({antialias: true});
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({color: 0x00ff00});
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;

    function animate() {
        requestAnimationFrame(animate);

        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;

        renderer.render(scene, camera);
    }
    animate();
}

// Create a 2D game environment using Phaser
function initPhaserGame() {
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
        this.load.image('sky', 'path/to/sky.png');
        this.load.image('ground', 'path/to/ground.png');
        this.load.image('star', 'path/to/star.png');
    }

    function create() {
        this.add.image(400, 300, 'sky');
    }

    function update() {
        // Game update logic
    }
}

// Initialize both Three.js and Phaser environments
initThreeJsScene();
initPhaserGame();