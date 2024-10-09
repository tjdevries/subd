document.addEventListener('DOMContentLoaded', () => {
    // Add a 3D animated title using three.js
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.TorusGeometry(0.7, 0.3, 16, 100);
    const material = new THREE.MeshBasicMaterial({ color: 0x0077ff });
    const torus = new THREE.Mesh(geometry, material);
    scene.add(torus);

    camera.position.z = 5;

    const animate = function () {
        requestAnimationFrame(animate);

        torus.rotation.x += 0.01;
        torus.rotation.y += 0.01;

        renderer.render(scene, camera);
    };

    animate();

    // Add audio visualization using phaser.js
    const config = {
        type: Phaser.AUTO,
        width: 800,
        height: 600,
        scene: {
            preload: preload,
            create: create
        }
    };

    const game = new Phaser.Game(config);

    function preload() {
        this.load.audio('song', '/songs/73bf39ca-c2c4-43db-9cc6-77d89f9219f5.mp3');
    }

    function create() {
        const music = this.sound.add('song');
        music.play();

        const graphics = this.add.graphics();
        const frequencyData = new Uint8Array(256);

        this.sound.context.resume();
        const analyser = this.sound.context.createAnalyser();
        analyser.fftSize = 512;
        analyser.connect(this.sound.context.destination);

        const source = this.sound.context.createMediaElementSource(music.source.buffer.control);
        source.connect(analyser);

        this.time.addEvent({
            delay: 50,
            loop: true,
            callback: () => {
                analyser.getByteFrequencyData(frequencyData);

                graphics.clear();
                graphics.fillStyle(0xffffff, 1);

                for (let i = 0; i < frequencyData.length; i++) {
                    let barHeight = frequencyData[i];
                    graphics.fillRect(i * 3, 600 - barHeight, 2, barHeight);
                }
            }
        });
    }
});