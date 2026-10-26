/**
 * KeyboardShortcutController
 *
 * Implements professional single-key desktop muscle memory with strict
 * first-responder shielding against active text fields, conforming to
 * First Proof Desktop Keyboard Architecture invariants.
 *
 * Single-key shortcuts (1..5 for ratings, P/X/U for flags, C for crop,
 * Z for zoom, [/] for rotation, \ for original peek) must NEVER fire
 * while the user is actively typing in editable text fields.
 */

export interface KeyboardActions {
  onEscape: () => void;
  onSelectTab: (tab: string) => void;
  onResetAllEdits: () => void;
  onApplyAutoTone: () => void;
  onOpenExportModal: () => void;
  onToggleMetadata: () => void;
  onSetRating: (rating: number) => void;
  onTogglePick: () => void;
  onToggleReject: () => void;
  onUnflag: () => void;
  onEnter: () => void;
  onToggleCrop: () => void;
  onCropFlipOrientation?: () => void;
  onTogglePixelPeep: () => void;
  onRotate: (deltaDeg: number) => void;
  onToggleEyedropper: (mode: 'wb') => void;
  onPeekOriginal: (active: boolean) => void;
  onNavigate: (direction: -1 | 1) => void;
  onToggleClippingOverlay?: () => void;
  onToggleHistogram?: () => void;
  isCropActive: () => boolean;
}

export function isTextInputActive(): boolean {
  const activeEl = document.activeElement;
  if (!activeEl) return false;
  const isInput =
    activeEl.tagName === 'INPUT' &&
    (activeEl as HTMLInputElement).type !== 'range';
  const isText =
    activeEl.tagName === 'TEXTAREA' ||
    (activeEl as HTMLElement).isContentEditable;
  return Boolean(isInput || isText);
}

export class KeyboardShortcutController {
  private actions: KeyboardActions;
  private isBound: boolean = false;
  private keydownHandler: (e: KeyboardEvent) => void;
  private keyupHandler: (e: KeyboardEvent) => void;
  private blurHandler: () => void;

  constructor(actions: KeyboardActions) {
    this.actions = actions;

    this.keydownHandler = (e: KeyboardEvent) => this.handleKeyDown(e);
    this.keyupHandler = (e: KeyboardEvent) => this.handleKeyUp(e);
    this.blurHandler = () => this.handleBlur();
  }

  public bind(): void {
    if (this.isBound) return;
    window.addEventListener('keydown', this.keydownHandler);
    window.addEventListener('keyup', this.keyupHandler);
    window.addEventListener('blur', this.blurHandler);
    this.isBound = true;
  }

  public unbind(): void {
    if (!this.isBound) return;
    window.removeEventListener('keydown', this.keydownHandler);
    window.removeEventListener('keyup', this.keyupHandler);
    window.removeEventListener('blur', this.blurHandler);
    this.isBound = false;
  }

  private handleKeyDown(e: KeyboardEvent): void {
    // 1. Escape key handling (always handles dismiss/rollback regardless of text focus)
    if (e.key === 'Escape') {
      this.actions.onEscape();
      return;
    }

    // 2. Strict First-Responder Shielding
    if (isTextInputActive()) {
      return;
    }

    // 3. Modifier-based shortcuts
    if ((e.metaKey || e.ctrlKey) && !e.shiftKey) {
      if (e.key === '1') {
        e.preventDefault();
        this.actions.onSelectTab('develop');
        return;
      }
      if (e.key === '2') {
        e.preventDefault();
        this.actions.onSelectTab('emulsion');
        return;
      }
      if (e.key === '3') {
        e.preventDefault();
        this.actions.onSelectTab('detail');
        return;
      }
      if (e.key === 'e' || e.key === 'E') {
        e.preventDefault();
        this.actions.onOpenExportModal();
        return;
      }
      if (e.key === 'u' || e.key === 'U') {
        e.preventDefault();
        this.actions.onApplyAutoTone();
        return;
      }
    }

    if (
      (e.metaKey || e.ctrlKey) &&
      e.shiftKey &&
      (e.key === 'r' || e.key === 'R')
    ) {
      e.preventDefault();
      this.actions.onResetAllEdits();
      return;
    }

    // 4. Single-key muscle memory
    switch (e.key) {
      case '1':
      case '2':
      case '3':
      case '4':
      case '5':
        this.actions.onSetRating(parseInt(e.key, 10));
        break;
      case '0':
        this.actions.onSetRating(0);
        break;
      case 'p':
      case 'P':
        this.actions.onTogglePick();
        break;
      case 'x':
      case 'X':
        if (this.actions.isCropActive() && this.actions.onCropFlipOrientation) {
          this.actions.onCropFlipOrientation();
          return;
        }
        this.actions.onToggleReject();
        break;
      case 'u':
      case 'U':
        this.actions.onUnflag();
        break;
      case 'Enter':
        this.actions.onEnter();
        break;
      case 'c':
      case 'C':
        this.actions.onToggleCrop();
        break;
      case 'z':
      case 'Z':
        if (!this.actions.isCropActive()) {
          this.actions.onTogglePixelPeep();
        }
        break;
      case 'i':
      case 'I':
        this.actions.onToggleMetadata();
        break;
      case 'j':
      case 'J':
        if (this.actions.onToggleClippingOverlay) {
          this.actions.onToggleClippingOverlay();
        }
        break;
      case 'h':
      case 'H':
        if (this.actions.onToggleHistogram) {
          this.actions.onToggleHistogram();
        }
        break;
      case '[':
        this.actions.onRotate(-90);
        break;
      case ']':
        this.actions.onRotate(90);
        break;
      case 'w':
      case 'W':
        this.actions.onToggleEyedropper('wb');
        break;
      case '\\':
        this.actions.onPeekOriginal(true);
        break;
      case 'ArrowLeft':
      case 'ArrowUp':
        e.preventDefault();
        this.actions.onNavigate(-1);
        break;
      case 'ArrowRight':
      case 'ArrowDown':
        e.preventDefault();
        this.actions.onNavigate(1);
        break;
    }
  }

  private handleKeyUp(e: KeyboardEvent): void {
    if (e.key === '\\') {
      this.actions.onPeekOriginal(false);
    }
  }

  private handleBlur(): void {
    this.actions.onPeekOriginal(false);
  }
}
