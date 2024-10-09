// Import necessary libraries
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Phaser from 'phaser';

// Three.js setup
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
camera.position.z = 5;
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Lighting
const ambientLight = new THREE.AmbientLight(0x404040);
scene.add(ambientLight);
const pointLight = new THREE.PointLight(0xffffff, 1, 100);
pointLight.position.set(10, 10, 10);
scene.add(pointLight);

// Load a 3D model
const loader = new GLTFLoader();
loader.load('/path/to/deez_nuts_model.glb', function (gltf) {
    scene.add(gltf.scene);
    animate3D();
}, undefined, function (error) {
    console.error('An error occurred while loading the model.', error);
});

// Torus animation
const geometry = new THREE.TorusGeometry(1, 0.4, 16, 100);
const material = new THREE.MeshStandardMaterial({ color: 0xff6347 });
const torus = new THREE.Mesh(geometry, material);
scene.add(torus);

torus.rotation.x += 0.01;
torus.rotation.y += 0.01;

const controls = new OrbitControls(camera, renderer.domElement);
controls.update();

function animate3D() {
    requestAnimationFrame(animate3D);
    torus.rotation.x += 0.01;
    torus.rotation.y += 0.01;
    controls.update();
    renderer.render(scene, camera);
}
animate3D();

// Phaser.js setup
var phaserConfig = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    physics: {
        default: 'arcade',
        arcade: {
            gravity: { y: 200 }
        }
    },
    scene: {
        preload: phaserPreload,
        create: phaserCreate,
        update: phaserUpdate
    }
};

var game = new Phaser.Game(phaserConfig);

function phaserPreload() {
    this.load.image('sky', 'assets/sky.png');
    this.load.image('star', 'assets/star.png');
}

function phaserCreate() {
    this.add.image(400, 300, 'sky');
    var stars = this.physics.add.group({
        key: 'star',
        repeat: 11,
        setXY: { x: 12, y: 0, stepX: 70 }
    });

    stars.children.iterate(function (child) {
        child.setBounceY(Phaser.Math.FloatBetween(0.4, 0.8));
    });

    this.physics.add.collider(stars, stars);
}

function phaserUpdate() {}
