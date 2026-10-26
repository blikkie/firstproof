import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  isTextInputActive,
  type KeyboardActions,
  KeyboardShortcutController,
} from '../controllers/KeyboardShortcutController';

describe('KeyboardShortcutController', () => {
  let actions: KeyboardActions;
  let controller: KeyboardShortcutController;

  beforeEach(() => {
    document.body.innerHTML = '';
    actions = {
      onEscape: vi.fn(),
      onSelectTab: vi.fn(),
      onResetAllEdits: vi.fn(),
      onApplyAutoTone: vi.fn(),
      onOpenExportModal: vi.fn(),
      onToggleMetadata: vi.fn(),
      onSetRating: vi.fn(),
      onTogglePick: vi.fn(),
      onToggleReject: vi.fn(),
      onUnflag: vi.fn(),
      onEnter: vi.fn(),
      onToggleCrop: vi.fn(),
      onCropFlipOrientation: vi.fn(),
      onTogglePixelPeep: vi.fn(),
      onRotate: vi.fn(),
      onToggleEyedropper: vi.fn(),
      onPeekOriginal: vi.fn(),
      onNavigate: vi.fn(),
      onToggleClippingOverlay: vi.fn(),
      onToggleHistogram: vi.fn(),
      isCropActive: vi.fn().mockReturnValue(false),
    };

    controller = new KeyboardShortcutController(actions);
    controller.bind();
  });

  afterEach(() => {
    controller.unbind();
    document.body.innerHTML = '';
    vi.restoreAllMocks();
  });

  describe('Single-Key Darkroom Shortcuts', () => {
    it('sets rating with keys 1 to 5 and clears with 0', () => {
      for (let r = 1; r <= 5; r++) {
        window.dispatchEvent(new KeyboardEvent('keydown', { key: String(r) }));
        expect(actions.onSetRating).toHaveBeenCalledWith(r);
      }
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '0' }));
      expect(actions.onSetRating).toHaveBeenCalledWith(0);
    });

    it('sets pick, reject, and unflag status with P, X, U', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'p' }));
      expect(actions.onTogglePick).toHaveBeenCalledTimes(1);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'x' }));
      expect(actions.onToggleReject).toHaveBeenCalledTimes(1);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'u' }));
      expect(actions.onUnflag).toHaveBeenCalledTimes(1);
    });

    it('toggles crop mode with C and zoom with Z', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'c' }));
      expect(actions.onToggleCrop).toHaveBeenCalledTimes(1);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'z' }));
      expect(actions.onTogglePixelPeep).toHaveBeenCalledTimes(1);
    });

    it('toggles clipping overlay with J', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'j' }));
      expect(actions.onToggleClippingOverlay).toHaveBeenCalledTimes(1);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'J' }));
      expect(actions.onToggleClippingOverlay).toHaveBeenCalledTimes(2);
    });

    it('toggles histogram with H', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'h' }));
      expect(actions.onToggleHistogram).toHaveBeenCalledTimes(1);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'H' }));
      expect(actions.onToggleHistogram).toHaveBeenCalledTimes(2);
    });

    it('rotates canvas with [ and ]', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '[' }));
      expect(actions.onRotate).toHaveBeenCalledWith(-90);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: ']' }));
      expect(actions.onRotate).toHaveBeenCalledWith(90);
    });

    it('triggers original peek comparison with backslash on keydown and keyup', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '\\' }));
      expect(actions.onPeekOriginal).toHaveBeenCalledWith(true);

      window.dispatchEvent(new KeyboardEvent('keyup', { key: '\\' }));
      expect(actions.onPeekOriginal).toHaveBeenCalledWith(false);
    });

    it('navigates photos with ArrowLeft and ArrowRight', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft' }));
      expect(actions.onNavigate).toHaveBeenCalledWith(-1);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
      expect(actions.onNavigate).toHaveBeenCalledWith(1);
    });
  });

  describe('Desktop Keyboard Architecture & First-Responder Shielding', () => {
    it('strictly suppresses all shortcuts when typing in a text input', () => {
      const input = document.createElement('input');
      input.type = 'text';
      document.body.appendChild(input);
      input.focus();

      expect(isTextInputActive()).toBe(true);

      // Try firing single-key shortcuts
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '1' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '5' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'p' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'x' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'c' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'z' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '[' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: ']' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '\\' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft' }));

      // Invariant: ZERO darkroom mutations while typing
      expect(actions.onSetRating).not.toHaveBeenCalled();
      expect(actions.onTogglePick).not.toHaveBeenCalled();
      expect(actions.onToggleReject).not.toHaveBeenCalled();
      expect(actions.onToggleCrop).not.toHaveBeenCalled();
      expect(actions.onTogglePixelPeep).not.toHaveBeenCalled();
      expect(actions.onRotate).not.toHaveBeenCalled();
      expect(actions.onPeekOriginal).not.toHaveBeenCalled();
      expect(actions.onNavigate).not.toHaveBeenCalled();
    });

    it('strictly suppresses shortcuts when typing in a textarea or contenteditable element', () => {
      const textarea = document.createElement('textarea');
      document.body.appendChild(textarea);
      textarea.focus();

      expect(isTextInputActive()).toBe(true);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: '3' }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'p' }));

      expect(actions.onSetRating).not.toHaveBeenCalled();
      expect(actions.onTogglePick).not.toHaveBeenCalled();
    });

    it('always executes Escape even when a text field is focused', () => {
      const input = document.createElement('input');
      document.body.appendChild(input);
      input.focus();

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
      expect(actions.onEscape).toHaveBeenCalledTimes(1);
    });

    it('resumes shortcuts once the text field is blurred', () => {
      const input = document.createElement('input');
      document.body.appendChild(input);
      input.focus();
      input.blur();

      expect(isTextInputActive()).toBe(false);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: '4' }));
      expect(actions.onSetRating).toHaveBeenCalledWith(4);
    });
  });

  describe('Modifier-Based Shortcuts', () => {
    it('switches tabs with Cmd/Ctrl + 1..3', () => {
      const tabs = ['develop', 'emulsion', 'detail'];
      tabs.forEach((tab, idx) => {
        window.dispatchEvent(
          new KeyboardEvent('keydown', {
            key: String(idx + 1),
            metaKey: true,
          }),
        );
        expect(actions.onSelectTab).toHaveBeenCalledWith(tab);
      });
    });

    it('toggles metadata with single key I', () => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'i' }));
      expect(actions.onToggleMetadata).toHaveBeenCalledTimes(1);

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'I' }));
      expect(actions.onToggleMetadata).toHaveBeenCalledTimes(2);
    });

    it('opens export modal with Cmd/Ctrl + E', () => {
      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'e', metaKey: true }),
      );
      expect(actions.onOpenExportModal).toHaveBeenCalledTimes(1);
    });

    it('resets edits with Cmd/Ctrl + Shift + R', () => {
      window.dispatchEvent(
        new KeyboardEvent('keydown', {
          key: 'r',
          metaKey: true,
          shiftKey: true,
        }),
      );
      expect(actions.onResetAllEdits).toHaveBeenCalledTimes(1);
    });
  });
});
