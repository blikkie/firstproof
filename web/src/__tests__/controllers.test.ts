import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  isTextInputActive,
  type KeyboardActions,
  KeyboardShortcutController,
} from '../controllers/KeyboardShortcutController';

describe('Controller & First-Responder Architecture', () => {
  beforeEach(() => {
    document.body.innerHTML = '';
  });

  afterEach(() => {
    document.body.innerHTML = '';
  });

  describe('isTextInputActive First-Responder Detection', () => {
    it('identifies standard text input fields as active', () => {
      const textInput = document.createElement('input');
      textInput.type = 'text';
      document.body.appendChild(textInput);
      textInput.focus();

      expect(isTextInputActive()).toBe(true);
    });

    it('identifies search and email inputs as active text fields', () => {
      const searchInput = document.createElement('input');
      searchInput.type = 'search';
      document.body.appendChild(searchInput);
      searchInput.focus();

      expect(isTextInputActive()).toBe(true);
    });

    it('does NOT shield range slider inputs (sliders should allow keyboard navigation)', () => {
      const rangeInput = document.createElement('input');
      rangeInput.type = 'range';
      document.body.appendChild(rangeInput);
      rangeInput.focus();

      expect(isTextInputActive()).toBe(false);
    });

    it('identifies textarea and contenteditable elements as active', () => {
      const textarea = document.createElement('textarea');
      document.body.appendChild(textarea);
      textarea.focus();

      expect(isTextInputActive()).toBe(true);

      textarea.blur();
      const div = document.createElement('div');
      div.contentEditable = 'true';
      document.body.appendChild(div);
      div.focus();

      expect(isTextInputActive()).toBe(true);
    });

    it('does NOT shield regular buttons or canvas', () => {
      const button = document.createElement('button');
      document.body.appendChild(button);
      button.focus();

      expect(isTextInputActive()).toBe(false);

      const canvas = document.createElement('canvas');
      document.body.appendChild(canvas);
      canvas.focus();

      expect(isTextInputActive()).toBe(false);
    });
  });

  describe('KeyboardShortcutController Bind Lifecycle', () => {
    it('unbind removes all listeners and stops intercepting events', () => {
      const actions: KeyboardActions = {
        onEscape: vi.fn(),
        onSelectTab: vi.fn(),
        onResetAllEdits: vi.fn(),
        onApplyAutoTone: vi.fn(),
        onOpenExportModal: vi.fn(),
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
        isCropActive: vi.fn().mockReturnValue(false),
      };

      const controller = new KeyboardShortcutController(actions);
      controller.bind();

      window.dispatchEvent(new KeyboardEvent('keydown', { key: '5' }));
      expect(actions.onSetRating).toHaveBeenCalledWith(5);

      // Unbind and fire again
      controller.unbind();
      window.dispatchEvent(new KeyboardEvent('keydown', { key: '4' }));
      expect(actions.onSetRating).not.toHaveBeenCalledWith(4);
    });
  });
});
