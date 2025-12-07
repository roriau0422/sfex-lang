// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="introduction.html">Introduction</a></span></li><li class="chapter-item expanded "><li class="part-title">Getting Started</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="getting-started/installation.html"><strong aria-hidden="true">1.</strong> Installation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="getting-started/quick-start.html"><strong aria-hidden="true">2.</strong> Quick Start</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="getting-started/first-program.html"><strong aria-hidden="true">3.</strong> Your First Program</a></span></li><li class="chapter-item expanded "><li class="part-title">Core Concepts</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="core/why-sfx.html"><strong aria-hidden="true">4.</strong> Why SFX?</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="core/mathematical-honesty.html"><strong aria-hidden="true">5.</strong> Mathematical Honesty</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="core/one-based-indexing.html"><strong aria-hidden="true">6.</strong> 1-Based Indexing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="core/no-null.html"><strong aria-hidden="true">7.</strong> No Null Pointers</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="core/grapheme-clustering.html"><strong aria-hidden="true">8.</strong> Grapheme Clustering</a></span></li><li class="chapter-item expanded "><li class="part-title">Language Syntax</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="syntax/basics.html"><strong aria-hidden="true">9.</strong> Basic Syntax</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="syntax/variables.html"><strong aria-hidden="true">10.</strong> Variables and Assignment</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="syntax/data-types.html"><strong aria-hidden="true">11.</strong> Data Types</a><a class="chapter-fold-toggle"><div>❱</div></a></span><ol class="section"><li class="chapter-item "><span class="chapter-link-wrapper"><a href="syntax/types/numbers.html"><strong aria-hidden="true">11.1.</strong> Numbers</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="syntax/types/fastnumber.html"><strong aria-hidden="true">11.2.</strong> FastNumber</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="syntax/types/strings.html"><strong aria-hidden="true">11.3.</strong> Strings</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="syntax/types/boolean.html"><strong aria-hidden="true">11.4.</strong> Boolean</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="syntax/types/lists.html"><strong aria-hidden="true">11.5.</strong> Lists</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="syntax/types/maps.html"><strong aria-hidden="true">11.6.</strong> Maps</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="syntax/types/option.html"><strong aria-hidden="true">11.7.</strong> Option Types</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="syntax/types/weakref.html"><strong aria-hidden="true">11.8.</strong> Weak References</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="syntax/operators.html"><strong aria-hidden="true">12.</strong> Operators</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="syntax/comments.html"><strong aria-hidden="true">13.</strong> Comments</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="syntax/comma-separated.html"><strong aria-hidden="true">14.</strong> Comma-Separated Syntax</a></span></li><li class="chapter-item expanded "><li class="part-title">Control Flow</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="control-flow/if-else.html"><strong aria-hidden="true">15.</strong> If/Else</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="control-flow/when-otherwise.html"><strong aria-hidden="true">16.</strong> When/Is/Otherwise</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="control-flow/repeat-times.html"><strong aria-hidden="true">17.</strong> Repeat Times</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="control-flow/repeat-while.html"><strong aria-hidden="true">18.</strong> Repeat While</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="control-flow/for-each.html"><strong aria-hidden="true">19.</strong> For Each</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="control-flow/break-continue.html"><strong aria-hidden="true">20.</strong> Break and Continue</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="control-flow/return.html"><strong aria-hidden="true">21.</strong> Return</a></span></li><li class="chapter-item expanded "><li class="part-title">Object-Oriented Programming</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="oop/concepts.html"><strong aria-hidden="true">22.</strong> Concepts (Classes)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="oop/fields.html"><strong aria-hidden="true">23.</strong> Fields</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="oop/methods.html"><strong aria-hidden="true">24.</strong> Methods</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="oop/parameters.html"><strong aria-hidden="true">25.</strong> Method Parameters</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="oop/this.html"><strong aria-hidden="true">26.</strong> This Keyword</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="oop/instances.html"><strong aria-hidden="true">27.</strong> Creating Instances</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="oop/set.html"><strong aria-hidden="true">28.</strong> Set Statement</a></span></li><li class="chapter-item expanded "><li class="part-title">Context-Oriented Programming</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="cop/situations.html"><strong aria-hidden="true">29.</strong> Situations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="cop/adjustments.html"><strong aria-hidden="true">30.</strong> Adjustments</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="cop/switch.html"><strong aria-hidden="true">31.</strong> Switch On/Off</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="cop/proceed.html"><strong aria-hidden="true">32.</strong> Proceed</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="cop/use-cases.html"><strong aria-hidden="true">33.</strong> Use Cases</a></span></li><li class="chapter-item expanded "><li class="part-title">Reactive Programming</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reactive/when-observers.html"><strong aria-hidden="true">34.</strong> When Observers</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reactive/self-healing.html"><strong aria-hidden="true">35.</strong> Self-Healing Data</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reactive/cascading.html"><strong aria-hidden="true">36.</strong> Cascading Updates</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reactive/recursion.html"><strong aria-hidden="true">37.</strong> Recursion Guard</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reactive/use-cases.html"><strong aria-hidden="true">38.</strong> Use Cases</a></span></li><li class="chapter-item expanded "><li class="part-title">Error Handling</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="error-handling/try-catch.html"><strong aria-hidden="true">39.</strong> Try/Catch/Always</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="error-handling/errors.html"><strong aria-hidden="true">40.</strong> Error Objects</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="error-handling/best-practices.html"><strong aria-hidden="true">41.</strong> Best Practices</a></span></li><li class="chapter-item expanded "><li class="part-title">Concurrency</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="concurrency/background.html"><strong aria-hidden="true">42.</strong> Do in Background</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="concurrency/tasks.html"><strong aria-hidden="true">43.</strong> Task Handles</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="concurrency/channels.html"><strong aria-hidden="true">44.</strong> Channels</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="concurrency/thread-safety.html"><strong aria-hidden="true">45.</strong> Thread Safety</a></span></li><li class="chapter-item expanded "><li class="part-title">Standard Library</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/overview.html"><strong aria-hidden="true">46.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/file.html"><strong aria-hidden="true">47.</strong> File Operations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/data.html"><strong aria-hidden="true">48.</strong> Data Parsing</a><a class="chapter-fold-toggle"><div>❱</div></a></span><ol class="section"><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/json.html"><strong aria-hidden="true">48.1.</strong> JSON</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/xml.html"><strong aria-hidden="true">48.2.</strong> XML</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/html.html"><strong aria-hidden="true">48.3.</strong> HTML</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/csv.html"><strong aria-hidden="true">48.4.</strong> CSV</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/toml.html"><strong aria-hidden="true">48.5.</strong> TOML</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/networking.html"><strong aria-hidden="true">49.</strong> Networking</a><a class="chapter-fold-toggle"><div>❱</div></a></span><ol class="section"><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/http.html"><strong aria-hidden="true">49.1.</strong> HTTP</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/websocket.html"><strong aria-hidden="true">49.2.</strong> WebSocket</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/tcp.html"><strong aria-hidden="true">49.3.</strong> TCP</a></span></li><li class="chapter-item "><span class="chapter-link-wrapper"><a href="stdlib/udp.html"><strong aria-hidden="true">49.4.</strong> UDP</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/system.html"><strong aria-hidden="true">50.</strong> System</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/env.html"><strong aria-hidden="true">51.</strong> Environment</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/time.html"><strong aria-hidden="true">52.</strong> Time</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/math.html"><strong aria-hidden="true">53.</strong> Math</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="stdlib/llm.html"><strong aria-hidden="true">54.</strong> LLM Integration</a></span></li><li class="chapter-item expanded "><li class="part-title">JIT Compilation</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="jit/overview.html"><strong aria-hidden="true">55.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="jit/how-it-works.html"><strong aria-hidden="true">56.</strong> How It Works</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="jit/profiling.html"><strong aria-hidden="true">57.</strong> Profiling System</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="jit/tiers.html"><strong aria-hidden="true">58.</strong> Optimization Tiers</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="jit/benchmarks.html"><strong aria-hidden="true">59.</strong> Performance Benchmarks</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="jit/features.html"><strong aria-hidden="true">60.</strong> Supported Features</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="jit/limitations.html"><strong aria-hidden="true">61.</strong> Limitations</a></span></li><li class="chapter-item expanded "><li class="part-title">Examples</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/hello-world.html"><strong aria-hidden="true">62.</strong> Hello World</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/math.html"><strong aria-hidden="true">63.</strong> Mathematical Operations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/lists.html"><strong aria-hidden="true">64.</strong> Working with Lists</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/oop.html"><strong aria-hidden="true">65.</strong> Object-Oriented Examples</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/cop.html"><strong aria-hidden="true">66.</strong> Context-Oriented Examples</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/reactive.html"><strong aria-hidden="true">67.</strong> Reactive Observers</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/file-io.html"><strong aria-hidden="true">68.</strong> File I/O</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/http.html"><strong aria-hidden="true">69.</strong> HTTP Requests</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/concurrency.html"><strong aria-hidden="true">70.</strong> Concurrency</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="examples/physics.html"><strong aria-hidden="true">71.</strong> Physics Simulation</a></span></li><li class="chapter-item expanded "><li class="part-title">Advanced Topics</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/performance.html"><strong aria-hidden="true">72.</strong> Performance Tips</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/memory.html"><strong aria-hidden="true">73.</strong> Memory Management</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/debugging.html"><strong aria-hidden="true">74.</strong> Debugging</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/testing.html"><strong aria-hidden="true">75.</strong> Testing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="advanced/project-structure.html"><strong aria-hidden="true">76.</strong> Project Structure</a></span></li><li class="chapter-item expanded "><li class="part-title">Reference</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/keywords.html"><strong aria-hidden="true">77.</strong> Keyword Reference</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/precedence.html"><strong aria-hidden="true">78.</strong> Operator Precedence</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/stdlib-api.html"><strong aria-hidden="true">79.</strong> Standard Library API</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/errors.html"><strong aria-hidden="true">80.</strong> Error Messages</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/cli.html"><strong aria-hidden="true">81.</strong> Command Line Interface</a></span></li><li class="chapter-item expanded "><li class="part-title">Contributing</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/guide.html"><strong aria-hidden="true">82.</strong> Contributing Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/code-of-conduct.html"><strong aria-hidden="true">83.</strong> Code of Conduct</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/security.html"><strong aria-hidden="true">84.</strong> Security Policy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/dev-setup.html"><strong aria-hidden="true">85.</strong> Development Setup</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing/architecture.html"><strong aria-hidden="true">86.</strong> Architecture</a></span></li><li class="chapter-item expanded "><li class="part-title">Appendix</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="appendix/comparison.html"><strong aria-hidden="true">87.</strong> Comparison with Other Languages</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="appendix/migration.html"><strong aria-hidden="true">88.</strong> Migration Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="appendix/faq.html"><strong aria-hidden="true">89.</strong> FAQ</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="appendix/glossary.html"><strong aria-hidden="true">90.</strong> Glossary</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="appendix/changelog.html"><strong aria-hidden="true">91.</strong> Changelog</a></span></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            if (link.href === current_page
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        const sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        const sidebarAnchorToggles = document.querySelectorAll('.chapter-fold-toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(el => {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);


// ---------------------------------------------------------------------------
// Support for dynamically adding headers to the sidebar.

(function() {
    // This is used to detect which direction the page has scrolled since the
    // last scroll event.
    let lastKnownScrollPosition = 0;
    // This is the threshold in px from the top of the screen where it will
    // consider a header the "current" header when scrolling down.
    const defaultDownThreshold = 150;
    // Same as defaultDownThreshold, except when scrolling up.
    const defaultUpThreshold = 300;
    // The threshold is a virtual horizontal line on the screen where it
    // considers the "current" header to be above the line. The threshold is
    // modified dynamically to handle headers that are near the bottom of the
    // screen, and to slightly offset the behavior when scrolling up vs down.
    let threshold = defaultDownThreshold;
    // This is used to disable updates while scrolling. This is needed when
    // clicking the header in the sidebar, which triggers a scroll event. It
    // is somewhat finicky to detect when the scroll has finished, so this
    // uses a relatively dumb system of disabling scroll updates for a short
    // time after the click.
    let disableScroll = false;
    // Array of header elements on the page.
    let headers;
    // Array of li elements that are initially collapsed headers in the sidebar.
    // I'm not sure why eslint seems to have a false positive here.
    // eslint-disable-next-line prefer-const
    let headerToggles = [];
    // This is a debugging tool for the threshold which you can enable in the console.
    let thresholdDebug = false;

    // Updates the threshold based on the scroll position.
    function updateThreshold() {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;

        // The number of pixels below the viewport, at most documentHeight.
        // This is used to push the threshold down to the bottom of the page
        // as the user scrolls towards the bottom.
        const pixelsBelow = Math.max(0, documentHeight - (scrollTop + windowHeight));
        // The number of pixels above the viewport, at least defaultDownThreshold.
        // Similar to pixelsBelow, this is used to push the threshold back towards
        // the top when reaching the top of the page.
        const pixelsAbove = Math.max(0, defaultDownThreshold - scrollTop);
        // How much the threshold should be offset once it gets close to the
        // bottom of the page.
        const bottomAdd = Math.max(0, windowHeight - pixelsBelow - defaultDownThreshold);
        let adjustedBottomAdd = bottomAdd;

        // Adjusts bottomAdd for a small document. The calculation above
        // assumes the document is at least twice the windowheight in size. If
        // it is less than that, then bottomAdd needs to be shrunk
        // proportional to the difference in size.
        if (documentHeight < windowHeight * 2) {
            const maxPixelsBelow = documentHeight - windowHeight;
            const t = 1 - pixelsBelow / Math.max(1, maxPixelsBelow);
            const clamp = Math.max(0, Math.min(1, t));
            adjustedBottomAdd *= clamp;
        }

        let scrollingDown = true;
        if (scrollTop < lastKnownScrollPosition) {
            scrollingDown = false;
        }

        if (scrollingDown) {
            // When scrolling down, move the threshold up towards the default
            // downwards threshold position. If near the bottom of the page,
            // adjustedBottomAdd will offset the threshold towards the bottom
            // of the page.
            const amountScrolledDown = scrollTop - lastKnownScrollPosition;
            const adjustedDefault = defaultDownThreshold + adjustedBottomAdd;
            threshold = Math.max(adjustedDefault, threshold - amountScrolledDown);
        } else {
            // When scrolling up, move the threshold down towards the default
            // upwards threshold position. If near the bottom of the page,
            // quickly transition the threshold back up where it normally
            // belongs.
            const amountScrolledUp = lastKnownScrollPosition - scrollTop;
            const adjustedDefault = defaultUpThreshold - pixelsAbove
                + Math.max(0, adjustedBottomAdd - defaultDownThreshold);
            threshold = Math.min(adjustedDefault, threshold + amountScrolledUp);
        }

        if (documentHeight <= windowHeight) {
            threshold = 0;
        }

        if (thresholdDebug) {
            const id = 'mdbook-threshold-debug-data';
            let data = document.getElementById(id);
            if (data === null) {
                data = document.createElement('div');
                data.id = id;
                data.style.cssText = `
                    position: fixed;
                    top: 50px;
                    right: 10px;
                    background-color: 0xeeeeee;
                    z-index: 9999;
                    pointer-events: none;
                `;
                document.body.appendChild(data);
            }
            data.innerHTML = `
                <table>
                  <tr><td>documentHeight</td><td>${documentHeight.toFixed(1)}</td></tr>
                  <tr><td>windowHeight</td><td>${windowHeight.toFixed(1)}</td></tr>
                  <tr><td>scrollTop</td><td>${scrollTop.toFixed(1)}</td></tr>
                  <tr><td>pixelsAbove</td><td>${pixelsAbove.toFixed(1)}</td></tr>
                  <tr><td>pixelsBelow</td><td>${pixelsBelow.toFixed(1)}</td></tr>
                  <tr><td>bottomAdd</td><td>${bottomAdd.toFixed(1)}</td></tr>
                  <tr><td>adjustedBottomAdd</td><td>${adjustedBottomAdd.toFixed(1)}</td></tr>
                  <tr><td>scrollingDown</td><td>${scrollingDown}</td></tr>
                  <tr><td>threshold</td><td>${threshold.toFixed(1)}</td></tr>
                </table>
            `;
            drawDebugLine();
        }

        lastKnownScrollPosition = scrollTop;
    }

    function drawDebugLine() {
        if (!document.body) {
            return;
        }
        const id = 'mdbook-threshold-debug-line';
        const existingLine = document.getElementById(id);
        if (existingLine) {
            existingLine.remove();
        }
        const line = document.createElement('div');
        line.id = id;
        line.style.cssText = `
            position: fixed;
            top: ${threshold}px;
            left: 0;
            width: 100vw;
            height: 2px;
            background-color: red;
            z-index: 9999;
            pointer-events: none;
        `;
        document.body.appendChild(line);
    }

    function mdbookEnableThresholdDebug() {
        thresholdDebug = true;
        updateThreshold();
        drawDebugLine();
    }

    window.mdbookEnableThresholdDebug = mdbookEnableThresholdDebug;

    // Updates which headers in the sidebar should be expanded. If the current
    // header is inside a collapsed group, then it, and all its parents should
    // be expanded.
    function updateHeaderExpanded(currentA) {
        // Add expanded to all header-item li ancestors.
        let current = currentA.parentElement;
        while (current) {
            if (current.tagName === 'LI' && current.classList.contains('header-item')) {
                current.classList.add('expanded');
            }
            current = current.parentElement;
        }
    }

    // Updates which header is marked as the "current" header in the sidebar.
    // This is done with a virtual Y threshold, where headers at or below
    // that line will be considered the current one.
    function updateCurrentHeader() {
        if (!headers || !headers.length) {
            return;
        }

        // Reset the classes, which will be rebuilt below.
        const els = document.getElementsByClassName('current-header');
        for (const el of els) {
            el.classList.remove('current-header');
        }
        for (const toggle of headerToggles) {
            toggle.classList.remove('expanded');
        }

        // Find the last header that is above the threshold.
        let lastHeader = null;
        for (const header of headers) {
            const rect = header.getBoundingClientRect();
            if (rect.top <= threshold) {
                lastHeader = header;
            } else {
                break;
            }
        }
        if (lastHeader === null) {
            lastHeader = headers[0];
            const rect = lastHeader.getBoundingClientRect();
            const windowHeight = window.innerHeight;
            if (rect.top >= windowHeight) {
                return;
            }
        }

        // Get the anchor in the summary.
        const href = '#' + lastHeader.id;
        const a = [...document.querySelectorAll('.header-in-summary')]
            .find(element => element.getAttribute('href') === href);
        if (!a) {
            return;
        }

        a.classList.add('current-header');

        updateHeaderExpanded(a);
    }

    // Updates which header is "current" based on the threshold line.
    function reloadCurrentHeader() {
        if (disableScroll) {
            return;
        }
        updateThreshold();
        updateCurrentHeader();
    }


    // When clicking on a header in the sidebar, this adjusts the threshold so
    // that it is located next to the header. This is so that header becomes
    // "current".
    function headerThresholdClick(event) {
        // See disableScroll description why this is done.
        disableScroll = true;
        setTimeout(() => {
            disableScroll = false;
        }, 100);
        // requestAnimationFrame is used to delay the update of the "current"
        // header until after the scroll is done, and the header is in the new
        // position.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                // Closest is needed because if it has child elements like <code>.
                const a = event.target.closest('a');
                const href = a.getAttribute('href');
                const targetId = href.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    threshold = targetElement.getBoundingClientRect().bottom;
                    updateCurrentHeader();
                }
            });
        });
    }

    // Takes the nodes from the given head and copies them over to the
    // destination, along with some filtering.
    function filterHeader(source, dest) {
        const clone = source.cloneNode(true);
        clone.querySelectorAll('mark').forEach(mark => {
            mark.replaceWith(...mark.childNodes);
        });
        dest.append(...clone.childNodes);
    }

    // Scans page for headers and adds them to the sidebar.
    document.addEventListener('DOMContentLoaded', function() {
        const activeSection = document.querySelector('#mdbook-sidebar .active');
        if (activeSection === null) {
            return;
        }

        const main = document.getElementsByTagName('main')[0];
        headers = Array.from(main.querySelectorAll('h2, h3, h4, h5, h6'))
            .filter(h => h.id !== '' && h.children.length && h.children[0].tagName === 'A');

        if (headers.length === 0) {
            return;
        }

        // Build a tree of headers in the sidebar.

        const stack = [];

        const firstLevel = parseInt(headers[0].tagName.charAt(1));
        for (let i = 1; i < firstLevel; i++) {
            const ol = document.createElement('ol');
            ol.classList.add('section');
            if (stack.length > 0) {
                stack[stack.length - 1].ol.appendChild(ol);
            }
            stack.push({level: i + 1, ol: ol});
        }

        // The level where it will start folding deeply nested headers.
        const foldLevel = 3;

        for (let i = 0; i < headers.length; i++) {
            const header = headers[i];
            const level = parseInt(header.tagName.charAt(1));

            const currentLevel = stack[stack.length - 1].level;
            if (level > currentLevel) {
                // Begin nesting to this level.
                for (let nextLevel = currentLevel + 1; nextLevel <= level; nextLevel++) {
                    const ol = document.createElement('ol');
                    ol.classList.add('section');
                    const last = stack[stack.length - 1];
                    const lastChild = last.ol.lastChild;
                    // Handle the case where jumping more than one nesting
                    // level, which doesn't have a list item to place this new
                    // list inside of.
                    if (lastChild) {
                        lastChild.appendChild(ol);
                    } else {
                        last.ol.appendChild(ol);
                    }
                    stack.push({level: nextLevel, ol: ol});
                }
            } else if (level < currentLevel) {
                while (stack.length > 1 && stack[stack.length - 1].level > level) {
                    stack.pop();
                }
            }

            const li = document.createElement('li');
            li.classList.add('header-item');
            li.classList.add('expanded');
            if (level < foldLevel) {
                li.classList.add('expanded');
            }
            const span = document.createElement('span');
            span.classList.add('chapter-link-wrapper');
            const a = document.createElement('a');
            span.appendChild(a);
            a.href = '#' + header.id;
            a.classList.add('header-in-summary');
            filterHeader(header.children[0], a);
            a.addEventListener('click', headerThresholdClick);
            const nextHeader = headers[i + 1];
            if (nextHeader !== undefined) {
                const nextLevel = parseInt(nextHeader.tagName.charAt(1));
                if (nextLevel > level && level >= foldLevel) {
                    const toggle = document.createElement('a');
                    toggle.classList.add('chapter-fold-toggle');
                    toggle.classList.add('header-toggle');
                    toggle.addEventListener('click', () => {
                        li.classList.toggle('expanded');
                    });
                    const toggleDiv = document.createElement('div');
                    toggleDiv.textContent = '❱';
                    toggle.appendChild(toggleDiv);
                    span.appendChild(toggle);
                    headerToggles.push(li);
                }
            }
            li.appendChild(span);

            const currentParent = stack[stack.length - 1];
            currentParent.ol.appendChild(li);
        }

        const onThisPage = document.createElement('div');
        onThisPage.classList.add('on-this-page');
        onThisPage.append(stack[0].ol);
        const activeItemSpan = activeSection.parentElement;
        activeItemSpan.after(onThisPage);
    });

    document.addEventListener('DOMContentLoaded', reloadCurrentHeader);
    document.addEventListener('scroll', reloadCurrentHeader, { passive: true });
})();

