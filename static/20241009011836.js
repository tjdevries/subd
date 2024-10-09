    document.addEventListener('DOMContentLoaded', function () {
        // Create the three.js scene
        const scene = new THREE.Scene();
        const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
        const renderer = new THREE.WebGLRenderer({ antialias: true });
        renderer.setSize(window.innerWidth, window.innerHeight);
        document.body.appendChild(renderer.domElement);

        // Create a rotating cube
        const geometry = new THREE.BoxGeometry();
        const material = new THREE.MeshBasicMaterial({ color: 0x00ff00, wireframe: true });
        const cube = new THREE.Mesh(geometry, material);
        scene.add(cube);
        camera.position.z = 5;

        const animateCube = function () {
            requestAnimationFrame(animateCube);
            cube.rotation.x += 0.01;
            cube.rotation.y += 0.01;
            renderer.render(scene, camera);
        };
        animateCube();

        // Load starry background
        const starGeometry = new THREE.SphereGeometry(50, 64, 64);
        const starMaterial = new THREE.MeshBasicMaterial({ map: new THREE.TextureLoader().load('stars.jpg'), side: THREE.BackSide });
        const stars = new THREE.Mesh(starGeometry, starMaterial);
        scene.add(stars);

        // Teletext animation with highlights in colors of the '90s
        const teletext = document.querySelectorAll('.unplayed_song a');
        teletext.forEach((song, index) => {
            song.style.transition = 'color 0.3s ease';
            song.addEventListener('mouseover', function () {
                this.style.color = `hsl(${Math.random() * 360}, 100%, 50%)`;
            });
            song.addEventListener('mouseout', function () {
                this.style.color = 'inherit';
            });
        });

        // Animate song links with bouncy text effect
        const songs = document.querySelectorAll('.song a');
        songs.forEach((song, index) => {
            song.style.transition = 'transform 0.2s ease';
            song.addEventListener('mouseover', function () {
                this.style.transform = 'scale(1.2)';
            });
            song.addEventListener('mouseout', function () {
                this.style.transform = 'scale(1)';
            });
        });

        // Phaser.js for pixelated game-like effects
        const config = {
            type: Phaser.AUTO,
            width: window.innerWidth,
            height: window.innerHeight,
            scene: {
                preload: preload,
                create: create,
                update: update
            }
        };

        const game = new Phaser.Game(config);
        function preload() {
            this.load.image('tile', 'path/to/tile.png');
        }
        function create() {
            this.add.image(config.width / 2, config.height / 2, 'tile');
        }
        function update() {}

        // Lighting and dynamic transition effects
        const header = document.querySelector('.header-container');
        header.style.transition = 'background-color 1500ms ease';
        setInterval(() => {
            header.style.backgroundColor = `hsl(${Math.random() * 360}, 100%, 50%)`;
        }, 3000);

    });