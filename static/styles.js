// styles.js
import * as THREE from 'three';
import { Scene, PerspectiveCamera, WebGLRenderer, BoxGeometry, MeshBasicMaterial, Mesh } from 'three';
import Phaser from 'phaser';
import Chart from 'chart.js/auto';
import p5 from 'p5';
import * as AFRAME from 'aframe';
import { zim } from 'zimjs';

// Three.js for 3D background animation
const scene = new Scene();
const camera = new PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

const geometry = new BoxGeometry();
const material = new MeshBasicMaterial({ color: 0x00ff00 });
const cube = new Mesh(geometry, material);
scene.add(cube);

camera.position.z = 5;

function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
}
animate();

// Phaser.js for interactive elements
const config = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    scene: {
        preload: function () {
            this.load.image('sky', 'assets/sky.png');
            this.load.image('star', 'assets/star.png');
        },
        create: function () {
            this.add.image(400, 300, 'sky');
            const particles = this.add.particles('star');
            const emitter = particles.createEmitter({
                speed: 100,
                scale: { start: 0.5, end: 0 },
                blendMode: 'ADD'
            });

            const logo = this.physics.add.image(400, 100, 'logo');
            logo.setVelocity(100, 200);
            logo.setBounce(1, 1);
            logo.setCollideWorldBounds(true);

            emitter.startFollow(logo);
        }
    }
};
const game = new Phaser.Game(config);

// Chart.js for dynamic charts
const ctx = document.getElementById('myChart').getContext('2d');
const myChart = new Chart(ctx, {
    type: 'bar',
    data: {
        labels: ['Red', 'Blue', 'Yellow', 'Green', 'Purple', 'Orange'],
        datasets: [{
            label: '# of Votes',
            data: [12, 19, 3, 5, 2, 3],
            backgroundColor: [
                'rgba(255, 99, 132, 0.2)',
                'rgba(54, 162, 235, 0.2)',
                'rgba(255, 206, 86, 0.2)',
                'rgba(75, 192, 192, 0.2)',
                'rgba(153, 102, 255, 0.2)',
                'rgba(255, 159, 64, 0.2)'
            ],
            borderColor: [
                'rgba(255, 99, 132, 1)',
                'rgba(54, 162, 235, 1)',
                'rgba(255, 206, 86, 1)',
                'rgba(75, 192, 192, 1)',
                'rgba(153, 102, 255, 1)',
                'rgba(255, 159, 64, 1)'
            ],
            borderWidth: 1
        }]
    },
    options: {
        scales: {
            y: {
                beginAtZero: true
            }
        }
    }
});

// P5.js for additional visual effects
new p5((sketch) => {
    sketch.setup = function() {
        sketch.createCanvas(400, 400);
    };

    sketch.draw = function() {
        sketch.background(200);
        sketch.fill(255);
        sketch.ellipse(sketch.mouseX, sketch.mouseY, 50, 50);
    };
}, 'p5-container');

// Aframe.io for 3D interactive elements
const aframeScene = document.createElement('a-scene');
const aframeBox = document.createElement('a-box');
aframeBox.setAttribute('position', '0 2 -5');
aframeBox.setAttribute('rotation', '0 45 0');
aframeBox.setAttribute('color', '#4CC3D9');
aframeScene.appendChild(aframeBox);
document.body.appendChild(aframeScene);

aframeScene.addEventListener('loaded', () => {
    console.log('A-Frame scene fully loaded');
});

// Zim.js for UI elements
const frame = new zim.Frame("fit", 1024, 768);
frame.on("ready", () => {
    const stage = frame.stage;
    const circle = new zim.Circle(50, "rgba(255, 255, 255, 0.5)");
    stage.addChild(circle);
    circle.center(stage);
    circle.drag();
});