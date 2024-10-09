// Import Three.js for 3D animations
import * as THREE from 'three';

// Set up a basic Three.js scene
function createScene() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Add some 3D objects
    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    // Position the camera
    camera.position.z = 5;

    // Animation loop
    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
    }

    animate();
}

// Initialize the scene on load
window.onload = createScene;

// Phaser.js configuration for interactive and game-like animations
import Phaser from 'phaser';

function createPhaserGame() {
    new Phaser.Game({
        type: Phaser.AUTO,
        width: 800,
        height: 600,
        scene: {
            preload: function () {
                this.load.setBaseURL('https://labs.phaser.io');
                this.load.image('sky', 'assets/skies/space3.png');
                this.load.image('star', 'assets/demoscene/star.png');
            },
            create: function () {
                this.add.image(400, 300, 'sky');
                const particles = this.add.particles('star');
                particles.createEmitter({
                    speed: 100,
                    scale: { start: 1, end: 0 },
                    blendMode: 'ADD'
                });
            },
        }
    });
}

window.onload = () => {
    createScene();
    createPhaserGame();
};