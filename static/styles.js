// Import the necessary libraries
import * as THREE from 'three';
import Phaser from 'phaser';

// Function to initialize Three.js scene
function initThree() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer({ alpha: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;

    function animate() {
        requestAnimationFrame(animate);

        // Rotate the cube for a dynamic effect
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;

        renderer.render(scene, camera);
    }
    animate();
}

// Function to create Phaser game
function initPhaser() {
    const config = {
        type: Phaser.AUTO,
        width: 800,
        height: 600,
        physics: {
            default: 'arcade',
            arcade: {
                gravity: { y: 200 },
                debug: false
            }
        },
        scene: {
            preload: preload,
            create: create,
            update: update
        }
    };

    const game = new Phaser.Game(config);

    function preload () {
        this.load.image('sky', 'assets/sky.png');
        this.load.image('star', 'assets/star.png');
    }

    function create () {
        this.add.image(400, 300, 'sky');

        const particles = this.add.particles('star');

        const emitter = particles.createEmitter({
            speed: 100,
            lifespan: {
                min: 4000,
                max: 6000
            },
            quantity: 2,
            scale: { start: 0.1, end: 0 },
            blendMode: 'ADD'
        });

        const logo = this.physics.add.image(400, 100, 'star');

        logo.setVelocity(100, 200);
        logo.setBounce(1, 1);
        logo.setCollideWorldBounds(true);

        emitter.startFollow(logo);
    }

    function update() {
        // Add custom updates as needed
    }
}

// Initialize both Three.js and Phaser
window.onload = function() {
    initThree();
    initPhaser();
};