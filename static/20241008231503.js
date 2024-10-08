// JavaScript to enhance interactivity and fun animations for 'AI Top of the Pops'.

document.addEventListener('DOMContentLoaded', function() {
    // Animate header text
    anime({
        targets: '.header',
        translateY: [-10, 0],
        opacity: [0, 1],
        duration: 1000,
        easing: 'easeOutExpo'
    });

    // Animated hover effect on nav links
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
      link.addEventListener('mouseover', () => {
          anime({
            targets: link,
            scale: 1.1,
            duration: 300
          });
      });
      link.addEventListener('mouseout', () => {
          anime({
            targets: link,
            scale: 1.0,
            duration: 300
          });
      });
    });

    // Expand and collapse effect on song sections
    const songSections = document.querySelectorAll('.unplayed_song, .current-song-info');
    songSections.forEach(section => {
      section.addEventListener('click', () => {
          anime({
            targets: section,
            height: section.clientHeight === 50 ? 100 : 50,
            duration: 500
          });
      });
    });

    // Rotating animation for images on hover
    const images = document.querySelectorAll('.ai_song_image img');
    images.forEach(img => {
      img.addEventListener('mouseover', () => {
          anime({
            targets: img,
            rotate: '1turn',
            duration: 800
          });
      });
    });
});

// CSS styles for enhanced appearance, place in styles.css
const cssContent = `
.header {
    color: #3498db;
    font-size: 36px;
    text-transform: uppercase;
}
.sub-header {
    color: #2980b9;
}
.nav-link {
    color: #34495e;
    text-decoration: none;
    margin: 0 15px;
    transition: color 0.3s;
}
.nav-link:hover {
    color: #e74c3c;
}
.unplayed_songs, .current-song-info {
    margin: 20px auto;
    padding: 10px;
    border: 1px solid #bdc3c7;
    border-radius: 8px;
    transition: height 0.3s ease-in-out;
}
.ai_song_image img {
    border-radius: 8px;
    transition: transform 0.3s;
}
.video {
    margin: 10px;
    border: 2px solid #ecf0f1;
    border-radius: 10px;
    overflow: hidden;
}
a {
    color: inherit;
    text-decoration: none;
}
a:hover {
    color: #e74c3c;
    text-decoration: underline;
}
`;

// Creating a Blob for the CSS file and triggering a download
downloadCSS('styles.css', cssContent);

function downloadCSS(filename, content) {
    const blob = new Blob([content], { type: 'text/css' });
    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();

    setTimeout(() => {
        document.body.removeChild(a);
        window.URL.revokeObjectURL(url);
    }, 100);
}