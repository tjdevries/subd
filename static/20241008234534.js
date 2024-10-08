document.addEventListener('DOMContentLoaded', function() {
    const header = document.querySelector('.header-container');
    const subHeaders = document.querySelectorAll('.sub-header');
    const navLinks = document.querySelectorAll('.nav-link');
    const songs = document.querySelectorAll('.unplayed_song, .song, .song-container');
    const pigColors = ['#FFB6C1', '#FFC0CB', '#FF69B4', '#FF1493'];
    const bounceAnimation = [
        { transform: 'translateY(0px)' },
        { transform: 'translateY(-10px)' },
        { transform: 'translateY(0px)' }
    ];
    const animationTiming = {
        duration: 1000,
        iterations: Infinity
    };

    // Animate header with a subtle bounce
    header.animate(bounceAnimation, Object.assign({}, animationTiming, { direction: 'alternate' }));
    
    // Animate sub-headers with color transition
    subHeaders.forEach((subHeader, index) => {
        subHeader.style.transition = 'color 3s ease-in-out';
        setInterval(() => {
            subHeader.style.color = pigColors[(index + Math.floor(Math.random() * pigColors.length)) % pigColors.length];
        }, 3000);
    });

    // Animate nav links with a fading effect
    navLinks.forEach(navLink => {
        navLink.style.transition = 'opacity 1s ease-in-out';
        navLink.addEventListener('mouseover', () => {
            navLink.style.opacity = '0.6';
        });
        navLink.addEventListener('mouseout', () => {
            navLink.style.opacity = '1.0';
        });
    });

    // Animate songs with a gentle fade in
    songs.forEach(song => {
        song.style.opacity = '0';
        song.style.transition = 'opacity 2s';
        song.getBoundingClientRect(); // trigger reflow
        song.style.opacity = '1';
    });
    
    // Add a page wide background animation
    document.body.style.background = 'linear-gradient(135deg, #FFD700, #FF4500)';
    document.body.style.backgroundSize = '400% 400%';
    document.body.style.animation = 'gradientAnimation 15s ease infinite';

    // Define keyframes for gradient animation
    const styleSheet = document.styleSheets[0];
    styleSheet.insertRule(`@keyframes gradientAnimation {
        0% {background-position: 0% 50%;}
        50% {background-position: 100% 50%;}
        100% {background-position: 0% 50%;}
    }`, styleSheet.cssRules.length);
});
/* styles.css */
.header-container {
    background-color: #FFFAF0;
    border-bottom: 3px solid #FFD700;
}

.sub-header {
    color: #FF7F50;
    font-size: 1.2em;
    margin: 5px 0;
}

.nav-link {
    display: inline-block;
    padding: 10px;
    font-weight: bold;
    color: #FFFFFF;
    background-color: #FF6347;
    text-decoration: none;
    border-radius: 20px;
    transition: background-color 0.3s ease;
}

.nav-link:hover {
    background-color: #FF4500;
}

.unplayed_songs, .current-song, .songs-container, .users-container {
    margin: 20px;
    padding: 15px;
    border-radius: 10px;
    background-color: #FFF8DC;
}

.song, .unplayed_song, .song-container {
    padding: 10px;
    margin-bottom: 10px;
    border: 1px solid #DAA520;
    border-radius: 5px;
    background-color: #FAFAD2;
}

.song:hover, .unplayed_song:hover, .song-container:hover {
    box-shadow: 0 0 10px #FFC0CB;
}

.image_scores, .videos {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-around;
}

.ai_song_image img {
    width: 150px;
    height: auto;
    border: 2px solid #FF69B4;
    border-radius: 10px;
    margin: 10px;
    transition: transform 0.3s ease;
}

.ai_song_image img:hover {
    transform: scale(1.1);
}

.video video {
    border: 2px solid #FF1493;
    border-radius: 10px;
    margin: 10px;
}

.lyrics {
    font-style: italic;
    color: #8B0000;
    margin: 20px;
}

body {
    margin: 0;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: #FFFAF0;
    transition: background 0.5s ease-in-out;
}/* Summary */
// The song evokes a nostalgic and mournful tone, highlighting a sense of loss and longing. It describes the absence of pigs, which once brought life, joy, and noise, leaving behind silence and ghosts of past memories. Fields that were once lively are now mute, permeated with a sorrowful emptiness.