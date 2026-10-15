export class PageNavController {
  private currentMode: 'overview' | 'darkroom' = 'overview';
  private onModeChangeCallback?: (mode: 'overview' | 'darkroom') => void;

  constructor(onModeChange?: (mode: 'overview' | 'darkroom') => void) {
    this.onModeChangeCallback = onModeChange;
    this.init();
  }

  private init(): void {
    // Check initial hash
    const hash = window.location.hash.toLowerCase();
    if (hash === '#darkroom') {
      this.setMode('darkroom', false);
    } else {
      this.setMode('overview', false);
    }

    // Listen to hash change
    window.addEventListener('hashchange', () => {
      const h = window.location.hash.toLowerCase();
      if (h === '#darkroom') {
        this.setMode('darkroom', false);
      } else if (h === '#overview' || !h) {
        this.setMode('overview', false);
      }
    });

    // Wire navigation tab buttons
    const btnOverview = document.getElementById('btn-nav-overview');
    const btnDarkroom = document.getElementById('btn-nav-darkroom');

    btnOverview?.addEventListener('click', (e) => {
      e.preventDefault();
      this.setMode('overview', true);
    });

    btnDarkroom?.addEventListener('click', (e) => {
      e.preventDefault();
      this.setMode('darkroom', true);
    });

    // Wire hero and overview CTA buttons
    const ctaDarkroom = document.getElementById('btn-hero-launch-darkroom');
    ctaDarkroom?.addEventListener('click', (e) => {
      e.preventDefault();
      this.setMode('darkroom', true);
    });

    const headerLaunch = document.getElementById('btn-header-launch-darkroom');
    headerLaunch?.addEventListener('click', (e) => {
      e.preventDefault();
      this.setMode('darkroom', true);
    });

    // Allow window-level dragover to automatically promote to darkroom
    window.addEventListener('dragover', (e) => {
      if (e.dataTransfer?.types.includes('Files')) {
        if (this.currentMode === 'overview') {
          this.setMode('darkroom', true);
        }
      }
    });
  }

  public setMode(mode: 'overview' | 'darkroom', updateHash = true): void {
    this.currentMode = mode;

    const editorialView = document.getElementById('view-editorial');
    const darkroomView = document.getElementById('view-darkroom');
    const filmstrip = document.getElementById('filmstrip-container');
    const darkroomHeaderActions = document.getElementById(
      'darkroom-header-actions',
    );
    const overviewHeaderNav = document.getElementById('overview-header-nav');

    const btnOverview = document.getElementById('btn-nav-overview');
    const btnDarkroom = document.getElementById('btn-nav-darkroom');

    if (mode === 'darkroom') {
      editorialView?.classList.add('hidden');
      darkroomView?.classList.remove('hidden');
      filmstrip?.classList.remove('hidden');
      darkroomHeaderActions?.classList.remove('hidden');
      overviewHeaderNav?.classList.add('hidden');

      btnOverview?.classList.remove('active');
      btnDarkroom?.classList.add('active');

      if (updateHash) {
        window.history.replaceState(null, '', '#darkroom');
      }

      // Notify renderer of geometry resize
      setTimeout(() => {
        window.dispatchEvent(new Event('resize'));
      }, 50);
    } else {
      editorialView?.classList.remove('hidden');
      darkroomView?.classList.add('hidden');
      filmstrip?.classList.add('hidden');
      darkroomHeaderActions?.classList.add('hidden');
      overviewHeaderNav?.classList.remove('hidden');

      btnOverview?.classList.add('active');
      btnDarkroom?.classList.remove('active');

      if (updateHash && window.location.hash === '#darkroom') {
        window.history.replaceState(null, '', '#overview');
      }
    }

    if (this.onModeChangeCallback) {
      this.onModeChangeCallback(mode);
    }
  }

  public getMode(): 'overview' | 'darkroom' {
    return this.currentMode;
  }
}
