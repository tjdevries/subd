// Import necessary libraries for animations
import * as THREE from 'three';
import Phaser from 'phaser';

// Function to initialize three.js animation
function initThreeJSScene() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Add a rotating cube
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

// Function to initialize Phaser game
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
        this.load.image('gift', 'path/to/gift.png'); // Load gift image/cube texture
    }

    function create() {
        this.add.text(200, 50, 'Give Me All Your Presents', { fontFamily: 'Arial', fontSize: 48, color: '#fff' });

        // Create a group of presents that are draggable
        const presents = this.physics.add.group({ key: 'gift', repeat: 5, setXY: { x: 12, y: 0, stepX: 70 } });

        presents.children.iterate(function (present) {
            present.setBounceY(Phaser.Math.FloatBetween(0.4, 0.8));
            present.setInteractive();
            present.on('pointerdown', () => {
                present.setTint(0xff0000);
                console.log('Gift opened!');
            });
        });
    }

    function update() {
        // Update logic
    }
}

window.onload = function() {
    initThreeJSScene(); // Initialize the 3D animated scene
    initPhaserGame(); // Initialize the 2D interactive game
};