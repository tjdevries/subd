document.addEventListener('DOMContentLoaded', function() {
    const header = document.querySelector('.header-container h1 a');
    header.style.transition = 'color 0.5s';
    header.addEventListener('mouseenter', function() {
        this.style.color = '#ff4081';
    });
    header.addEventListener('mouseleave', function() {
        this.style.color = '#fff';
    });
    
    const subHeaders = document.querySelectorAll('.sub-header');
    subHeaders.forEach(subHeader => {
        subHeader.style.transition = 'transform 0.3s ease-in-out';
        subHeader.addEventListener('mouseover', function() {
            this.style.transform = 'scale(1.1)';
        });
        subHeader.addEventListener('mouseout', function() {
            this.style.transform = 'scale(1)';
        });
    });

    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.style.transition = 'color 0.3s';
        link.addEventListener('mouseenter', function() {
            this.style.color = '#ff4081';
        });
        link.addEventListener('mouseleave', function() {
            this.style.color = '#ffffff';
        });
    });
    
    const songTitles = document.querySelectorAll('.grid-container .title, .songs-container .title');
    songTitles.forEach(title => {
        title.style.transition = 'opacity 0.5s';
        title.addEventListener('mouseenter', function() {
            this.style.opacity = '0.6';
        });
        title.addEventListener('mouseleave', function() {
            this.style.opacity = '1';
        });
    });
    
    const images = document.querySelectorAll('.ai_song_image img');
    images.forEach(image => {
        image.style.transition = 'transform 0.3s ease';
        image.addEventListener('mouseover', function() {
            this.style.transform = 'scale(1.1)';
        });
        image.addEventListener('mouseout', function() {
            this.style.transform = 'scale(1)';
        });
    });

    // Animation on scrolling
    const animateOnScroll = () => {
        const sections = document.querySelectorAll('section');
        const appearOptions = {
            threshold: 0.5,
            rootMargin: "0px 0px -50px 0px"
        };

        const appearOnScroll = new IntersectionObserver(function(entries, appearOnScroll) {
            entries.forEach(entry => {
                if (!entry.isIntersecting) {
                    return;
                } else {
                    entry.target.classList.add('appear');
                    appearOnScroll.unobserve(entry.target);
                }
            });
        }, appearOptions);

        sections.forEach(section => {
            appearOnScroll.observe(section);
        });
    }
    animateOnScroll();
    
    const saveButton = document.createElement('button');
    saveButton.textContent = 'Save Styles';
    saveButton.style.position = 'fixed';
    saveButton.style.bottom = '10px';
    saveButton.style.right = '10px';
    saveButton.style.padding = '10px 20px';
    saveButton.style.backgroundColor = '#ff4081';
    saveButton.style.border = 'none';
    saveButton.style.color = 'white';
    saveButton.style.cursor = 'pointer';
    saveButton.addEventListener('click', function() {
        const styles = `
            /* Styles for Animations */
            .appear {
                opacity: 1;
                transition: opacity 0.6s, transform 0.6s;
                transform: translateY(0);
            }
            section {
                opacity: 0;
                transform: translateY(20px);
            }
        `;
        const blob = new Blob([styles], { type: 'text/css' });
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = 'styles.css';
        link.click();
    });
    document.body.appendChild(saveButton);

});