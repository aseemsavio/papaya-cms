/* ==========================================================================
   Aperture — interaction layer
   Hero reveal, horizontal archive, virtualized tiles, post reading state.
   No dependencies. Everything is hosted with the theme.
   ========================================================================== */
(function () {
    "use strict";

    var doc = document;
    var root = doc.documentElement;
    var body = doc.body;
    var reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    var finePointer = window.matchMedia("(pointer: fine)").matches;

    function clamp(value, min, max) {
        return Math.min(max, Math.max(min, value));
    }

    function raf(fn) {
        return window.requestAnimationFrame(fn);
    }

    function $(selector, scope) {
        return (scope || doc).querySelector(selector);
    }

    /* ----------------------------------------------------------------------
       Header
       ---------------------------------------------------------------------- */

    var header = $("#site-head");

    /* ----------------------------------------------------------------------
       Hero — split the wordmark, fit it, reveal the cover
       ---------------------------------------------------------------------- */

    var hero = $("#hero");
    var heroWord = $(".hero-word");
    var heroImage = $(".hero-image");
    var heroImg = heroImage ? $("img", heroImage) : null;
    var heroChars = [];
    var heroReady = false;

    function splitWordmark() {
        if (!heroWord || heroWord.getAttribute("data-split") === "true") return;

        var text = (heroWord.textContent || "").trim();
        if (!text) return;

        heroWord.textContent = "";
        heroChars = [];

        for (var i = 0; i < text.length; i++) {
            var char = doc.createElement("span");
            char.className = "char";
            char.style.setProperty("--i", i);
            char.textContent = text.charAt(i) === " " ? "\u00A0" : text.charAt(i);
            char.setAttribute("aria-hidden", "true");
            heroWord.appendChild(char);
            heroChars.push(char);
        }

        heroWord.setAttribute("data-split", "true");
    }

    function fitWordmark() {
        if (!heroWord || !heroChars.length) return;

        var stage = heroWord.closest(".hero-stage");
        if (!stage) return;

        heroWord.style.fontSize = "";
        var available = stage.clientWidth - 28;
        if (available <= 0) return;

        var natural = heroWord.scrollWidth;
        if (natural > available) {
            var current = parseFloat(window.getComputedStyle(heroWord).fontSize) || 100;
            heroWord.style.fontSize = (current * available) / natural + "px";
        }
    }

    function revealHero() {
        if (heroReady) return;
        heroReady = true;
        body.classList.add("hero-loaded");
    }

    function initHero() {
        if (!hero) return;

        splitWordmark();
        // Fit once now and again when the display face settles.
        raf(function () {
            fitWordmark();
            if (doc.fonts && doc.fonts.ready) {
                doc.fonts.ready.then(fitWordmark).catch(function () {});
            }
        });

        if (heroImg) {
            if (heroImg.complete && heroImg.naturalWidth > 0) {
                heroImage.classList.add("is-loaded");
                raf(revealHero);
            } else {
                heroImg.addEventListener(
                    "load",
                    function () {
                        heroImage.classList.add("is-loaded");
                        revealHero();
                    },
                    { once: true }
                );
                heroImg.addEventListener("error", revealHero, { once: true });
                // Never let a slow network hold the page hostage.
                window.setTimeout(revealHero, 2200);
            }
        } else {
            revealHero();
        }
    }

    /* ----------------------------------------------------------------------
       Custom cursor
       ---------------------------------------------------------------------- */

    var cursor = null;
    var cursorLabel = null;
    var cursorX = -200;
    var cursorY = -200;
    var cursorRaf = 0;

    function initCursor() {
        if (!finePointer || reduceMotion) return;
        if ($(".gallery-pin") === null) return;

        cursor = doc.createElement("div");
        cursor.className = "cursor";
        cursor.setAttribute("aria-hidden", "true");
        cursorLabel = doc.createElement("span");
        cursorLabel.textContent = "View";
        cursor.appendChild(cursorLabel);
        doc.body.appendChild(cursor);

        doc.addEventListener(
            "mousemove",
            function (event) {
                cursorX = event.clientX;
                cursorY = event.clientY;
                if (!cursorRaf) {
                    cursorRaf = raf(function () {
                        cursorRaf = 0;
                        cursor.style.transform =
                            "translate3d(" + cursorX + "px," + cursorY + "px,0) translate(-50%,-50%)";
                    });
                }
            },
            { passive: true }
        );

        doc.addEventListener("mouseleave", function () {
            cursor.classList.remove("is-visible", "is-view");
        });
    }

    function setCursorState(visible, view) {
        if (!cursor) return;
        cursor.classList.toggle("is-visible", !!visible);
        cursor.classList.toggle("is-view", !!view);
    }

    /* ----------------------------------------------------------------------
       Gallery — the archive as a horizontal filmstrip
       ---------------------------------------------------------------------- */

    var gallery = $("#index");
    var pin = $("#gallery-pin");
    var track = $("#gallery-track");
    var endCard = $("#gallery-end");
    var progressBar = $("#gallery-progress-bar");
    var currentEl = $("#gallery-current");
    var hint = $("#gallery-hint");
    var dataEl = $("#gallery-data");

    var posts = [];
    try {
        posts = dataEl ? JSON.parse(dataEl.textContent) : [];
    } catch (error) {
        posts = [];
    }
    if (!Array.isArray(posts)) posts = [];

    var GAP = 16;
    var geometry = [];
    var totalWidth = 0;
    var scrollDistance = 0;
    var galleryTop = 0;
    var layoutVW = 0;
    var layoutVH = 0;
    var activeIndex = -1;
    var ticking = false;
    var galleryActive = false;
    var unloadDistance = 0;
    var currentScrollLeft = 0;

    function postUrl(post) {
        return "/posts/" + post.slug;
    }

    function imageUrl(post) {
        if (post.photo && post.photo.thumb) return post.photo.thumb;
        return post.cover_image || "";
    }

    function aspectOf(post) {
        var raw = post.photo ? parseFloat(post.photo.aspect) : 1.5;
        if (!isFinite(raw) || raw <= 0.2) return 1.5;
        return clamp(raw, 0.45, 3.2);
    }

    function colorOf(post) {
        return (post.photo && post.photo.color) || "#e9e6df";
    }

    function buildLayout() {
        if (!gallery || !pin || !track) return;

        layoutVW = window.innerWidth;
        layoutVH = window.innerHeight;

        var startX = layoutVW < 760 ? 2 : 8;
        var contentTop = layoutVW < 760 ? 88 : 100;
        var contentBottom = layoutVH - (layoutVW < 760 ? 76 : 88);
        var contentH = Math.max(320, contentBottom - contentTop);
        var gap = GAP;
        var rowOne = Math.round((contentH - gap) * 0.57);
        var rowTwo = contentH - gap - rowOne;

        var laneX = [startX, startX];
        geometry = [];
        totalWidth = startX;

        for (var i = 0; i < posts.length; i++) {
            var post = posts[i];
            var aspect = aspectOf(post);
            var feature = i % 7 === 3 && Math.abs(laneX[0] - laneX[1]) < 220;
            var x, y, w, h;

            if (feature) {
                x = Math.max(laneX[0], laneX[1]);
                h = contentH;
                w = clamp(h * aspect, Math.min(300, layoutVW * 0.5), layoutVW * 0.86);
                y = contentTop;
                laneX[0] = x + w + gap;
                laneX[1] = laneX[0];
            } else {
                var lane = laneX[0] <= laneX[1] ? 0 : 1;
                h = lane === 0 ? rowOne : rowTwo;
                w = clamp(h * aspect, Math.min(220, layoutVW * 0.36), layoutVW * 0.72);
                x = laneX[lane];
                y = lane === 0 ? contentTop : contentTop + rowOne + gap;
                laneX[lane] = x + w + gap;
            }

            geometry.push({ index: i, post: post, x: x, y: y, w: w, h: h, el: null, img: null });
        }

        var endX = Math.max(laneX[0], laneX[1]) + gap * 3;
        var endWidth = clamp(layoutVW * 0.46, 300, 660);

        if (endCard) {
            endCard.style.left = endX + "px";
            endCard.style.width = endWidth + "px";
            endCard.style.top = "0";
            endCard.style.height = "100%";
            endCard.style.display = "flex";
            endCard.style.visibility = "visible";
        }

        totalWidth = endX + endWidth;
        scrollDistance = Math.max(0, totalWidth - layoutVW);
        unloadDistance = layoutVW * 1.6;

        gallery.style.height = scrollDistance + layoutVH + "px";
        galleryTop = gallery.getBoundingClientRect().top + window.scrollY;

        for (var g = 0; g < geometry.length; g++) {
            geometry[g].el = null;
            geometry[g].img = null;
        }

        var tileNodes = track.querySelectorAll(".tile");
        for (var t = 0; t < tileNodes.length; t++) {
            tileNodes[t].parentNode.removeChild(tileNodes[t]);
        }

        updateGallery(true);
    }

    function makeTile(item) {
        var post = item.post;
        var a = doc.createElement("a");
        a.className = "tile";
        a.href = postUrl(post);
        a.style.setProperty("--tile-color", colorOf(post));
        a.style.width = item.w + "px";
        a.style.height = item.h + "px";
        a.style.transform =
            "translate3d(" + (item.x - currentScrollLeft) + "px," + item.y + "px,0)";
        a.setAttribute("aria-label", post.title || "View photograph");

        var media = doc.createElement("div");
        media.className = "tile-media";

        var img = doc.createElement("img");
        img.alt = post.title || "";
        img.decoding = "async";
        img.loading = item.index < 4 ? "eager" : "lazy";
        img.draggable = false;
        img.addEventListener("load", function () {
            a.classList.add("is-loaded");
        });
        img.addEventListener("error", function () {
            a.classList.add("is-loaded");
        });

        var url = imageUrl(post);
        if (url) img.src = url;
        if (img.complete && img.naturalWidth > 0) a.classList.add("is-loaded");

        media.appendChild(img);
        a.appendChild(media);

        var tag = doc.createElement("span");
        tag.className = "tile-tag";

        var title = doc.createElement("span");
        title.textContent = post.title || "Untitled";
        tag.appendChild(title);

        if (post.photo && post.photo.location) {
            var location = doc.createElement("span");
            location.className = "tile-arrow";
            location.textContent = post.photo.location;
            tag.appendChild(location);
        }

        var arrow = doc.createElement("span");
        arrow.className = "tile-arrow";
        arrow.textContent = "\u2197";
        tag.appendChild(arrow);

        a.appendChild(tag);
        track.appendChild(a);

        item.el = a;
        item.img = img;

        raf(function () {
            a.classList.add("is-in");
        });
    }

    function unmountTile(item) {
        if (!item.el) return;
        if (item.img) {
            item.img.removeAttribute("src");
            item.img = null;
        }
        if (item.el.parentNode) item.el.parentNode.removeChild(item.el);
        item.el.classList.remove("is-in", "is-loaded");
        item.el = null;
    }

    function updateGallery(force) {
        if (!gallery || !geometry.length) return;

        var y = window.scrollY;
        var raw = scrollDistance > 0 ? (y - galleryTop) / scrollDistance : 0;
        var progress = clamp(raw, 0, 1);
        var scrollLeft = progress * scrollDistance;
        currentScrollLeft = scrollLeft;

        var active =
            y >= galleryTop - layoutVH * 0.65 &&
            y <= galleryTop + scrollDistance + layoutVH * 0.35;

        if (active !== galleryActive || force) {
            galleryActive = active;
            body.classList.toggle("is-indexing", active);
            if (pin) pin.classList.toggle("has-cursor", active && !!cursor);
        }

        if (progressBar) progressBar.style.transform = "scaleX(" + progress + ")";

        if (hint && progress > 0.012) hint.classList.add("is-hidden");

        if (endCard) {
            endCard.style.transform = "translate3d(" + -scrollLeft + "px,0,0)";
        }

        var left = scrollLeft - layoutVW * 1.15;
        var right = scrollLeft + layoutVW * 2.15;

        for (var i = 0; i < geometry.length; i++) {
            var item = geometry[i];
            var visible = item.x + item.w > left && item.x < right;

            if (visible && !item.el) {
                makeTile(item);
            } else if (!visible && item.el) {
                var farLeft = item.x + item.w < scrollLeft - unloadDistance;
                var farRight = item.x > scrollLeft + layoutVW + unloadDistance;
                if (farLeft || farRight) unmountTile(item);
            }

            if (item.el) {
                item.el.style.transform =
                    "translate3d(" + (item.x - scrollLeft) + "px," + item.y + "px,0)";
            }
        }

        var probe = scrollLeft + layoutVW * 0.38;
        var current = 0;
        for (var c = 0; c < geometry.length; c++) {
            if (geometry[c].x + geometry[c].w * 0.5 <= probe) current = c;
        }
        if (progress >= 0.999) current = geometry.length - 1;

        if (current !== activeIndex) {
            activeIndex = current;
            if (currentEl) {
                currentEl.textContent = String(current + 1).padStart(2, "0");
            }
        }
    }

    function onGalleryClick(event) {
        var tile = event.target.closest ? event.target.closest("a.tile") : null;
        if (!tile) return;
        if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button !== 0) return;

        if (reduceMotion) return;

        var img = $("img", tile);
        var to = tile.getBoundingClientRect();
        var source = img && (img.currentSrc || img.src);
        if (!source) return;

        event.preventDefault();

        var overlay = doc.createElement("div");
        overlay.className = "page-transition";

        var clone = doc.createElement("img");
        clone.src = source;
        clone.alt = "";
        clone.style.left = to.left + "px";
        clone.style.top = to.top + "px";
        clone.style.width = to.width + "px";
        clone.style.height = to.height + "px";
        clone.style.transition =
            "left 0.72s cubic-bezier(0.16,1,0.3,1), top 0.72s cubic-bezier(0.16,1,0.3,1), width 0.72s cubic-bezier(0.16,1,0.3,1), height 0.72s cubic-bezier(0.16,1,0.3,1), filter 0.72s ease";

        overlay.appendChild(clone);
        doc.body.appendChild(overlay);
        body.classList.add("is-transitioning");

        var target = tile.getAttribute("href");

        raf(function () {
            raf(function () {
                clone.style.left = "0px";
                clone.style.top = "0px";
                clone.style.width = "100vw";
                clone.style.height = "100vh";
                clone.style.filter = "saturate(1)";
            });
        });

        var done = false;
        function go() {
            if (done) return;
            done = true;
            window.location.href = target;
        }

        window.setTimeout(go, 620);
        // A refresh or a slow image should still land on the post.
        window.setTimeout(go, 1500);
    }

    /* ----------------------------------------------------------------------
       Hero parallax
       ---------------------------------------------------------------------- */

    function parallaxHero() {
        if (!hero || !heroImage || reduceMotion) return;

        var y = window.scrollY;
        if (y > layoutVH * 1.25) return;

        var shift = y * 0.16;
        heroImage.style.translate = "0 " + shift + "px";
        heroImage.style.scale = String(1 + Math.min(y, layoutVH) * 0.00006);

        if (heroWord && heroChars.length) {
            heroWord.style.transform = "translate3d(0," + y * 0.055 + "px,0)";
            heroWord.style.opacity = String(clamp(1 - y / (layoutVH * 0.92), 0, 1));
        }
    }

    /* ----------------------------------------------------------------------
       Post page
       ---------------------------------------------------------------------- */

    var postPage = $(".post-page");
    var readingBar = $("#reading-progress-bar");
    var postScroll = $("#post-scroll");
    var postHero = $(".post-hero");
    var postReady = false;

    function finishPost() {
        if (postReady) return;
        postReady = true;
        body.classList.add("post-ready");
    }

    function initPost() {
        if (!postPage) return;

        if (postHero) {
            var img = $("img", postHero);
            if (!img || (img.complete && img.naturalWidth > 0)) {
                raf(finishPost);
            } else {
                img.addEventListener("load", function () {
                    raf(finishPost);
                });
                img.addEventListener("error", finishPost);
                window.setTimeout(finishPost, 2200);
            }
        } else {
            finishPost();
        }

        updateReading();
    }

    function updateReading() {
        if (!postPage) return;

        if (readingBar) {
            var height = doc.documentElement.scrollHeight - window.innerHeight;
            var p = height > 0 ? clamp(window.scrollY / height, 0, 1) : 0;
            readingBar.style.transform = "scaleX(" + p + ")";
        }

        if (postScroll) {
            postScroll.classList.toggle("is-hidden", window.scrollY > 48);
        }
    }

    /* ----------------------------------------------------------------------
       Scroll loop
       ---------------------------------------------------------------------- */

    function onScroll() {
        if (ticking) return;
        ticking = true;

        raf(function () {
            ticking = false;
            var y = window.scrollY;

            if (header) header.classList.toggle("is-scrolled", y > 8);

            if (gallery) {
                parallaxHero();
                updateGallery(false);
            } else if (postPage) {
                updateReading();
            }
        });
    }

    /* ----------------------------------------------------------------------
       Resize
       ---------------------------------------------------------------------- */

    var resizeTimer = 0;

    function onResize() {
        window.clearTimeout(resizeTimer);
        resizeTimer = window.setTimeout(function () {
            if (gallery) buildLayout();
            fitWordmark();
        }, 180);
    }

    /* ----------------------------------------------------------------------
       Boot
       ---------------------------------------------------------------------- */

    function boot() {
        initHero();
        initCursor();
        initPost();

        if (gallery) {
            buildLayout();
            track.addEventListener("click", onGalleryClick);

            if (pin) {
                pin.addEventListener("mouseenter", function () {
                    setCursorState(true, false);
                });
                pin.addEventListener("mouseleave", function () {
                    setCursorState(false, false);
                });
                pin.addEventListener("mousemove", function (event) {
                    var onTile = event.target.closest ? event.target.closest("a.tile") : null;
                    setCursorState(true, !!onTile);
                });
            }
        }

        window.addEventListener("scroll", onScroll, { passive: true });
        window.addEventListener("resize", onResize, { passive: true });
        window.addEventListener("orientationchange", onResize, { passive: true });

        onScroll();

        raf(function () {
            raf(function () {
                body.classList.add("is-ready");
            });
        });
    }

    if (doc.readyState === "loading") {
        doc.addEventListener("DOMContentLoaded", boot, { once: true });
    } else {
        boot();
    }
})();
