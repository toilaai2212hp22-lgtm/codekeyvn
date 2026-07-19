// CodeKey — Fcitx5 input method addon (Telex / VNI)
// Thin C++ wrapper around Rust libcodekey (include/codekey.h).

#include <fcitx/addonfactory.h>
#include <fcitx/addoninstance.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputmethodentry.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx-utils/capabilityflags.h>
#include <fcitx-utils/key.h>
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/textformatflags.h>
#include <fcitx-utils/utf8.h>
#include <fcitx/text.h>
#include <fcitx/userinterface.h>

#include "codekey.h"

#include <string>

namespace {

// KeyResult from Rust FFI:
// 0 Update, 1 Append, 2 CommitAndPass, 3 Commit, 4 Backspace, 5 Ignored
enum class KR : int {
    Update = 0,
    Append = 1,
    CommitAndPass = 2,
    Commit = 3,
    Backspace = 4,
    Ignored = 5,
};

class CodeKeyEngine final : public fcitx::InputMethodEngineV2 {
public:
    explicit CodeKeyEngine(fcitx::Instance *instance) : instance_(instance) {
        instance_->inputContextManager().registerProperty("codekeyState",
                                                          &factory_);
    }

    void keyEvent(const fcitx::InputMethodEntry &entry,
                  fcitx::KeyEvent &keyEvent) override {
        if (keyEvent.isRelease()) {
            return;
        }

        auto *ic = keyEvent.inputContext();
        if (!ic) {
            return;
        }

        auto *st = ensure(ic, entry);
        if (!st || !st->eng) {
            return;
        }

        const auto key = keyEvent.key();

        // Modifier combos → commit & let app handle
        if (key.states().testAny(fcitx::KeyStates{
                fcitx::KeyState::Ctrl, fcitx::KeyState::Alt,
                fcitx::KeyState::Super})) {
            flush(ic, st);
            return;
        }

        // Ignore pure modifiers
        if (key.isModifier()) {
            return;
        }

        if (key.check(FcitxKey_BackSpace)) {
            const int r = codekey_engine_backspace(st->eng);
            if (r == static_cast<int>(KR::Backspace)) {
                updateUI(ic, st);
                keyEvent.filterAndAccept();
            }
            return;
        }

        if (key.check(FcitxKey_Escape)) {
            codekey_engine_reset(st->eng);
            clearUI(ic);
            keyEvent.filterAndAccept();
            return;
        }

        if (key.check(FcitxKey_Return) || key.check(FcitxKey_KP_Enter)) {
            flush(ic, st);
            return; // let Enter pass to the app
        }

        // Map key → Unicode code point for the engine
        uint32_t ch = 0;
        if (key.sym() == FcitxKey_space) {
            ch = ' ';
        } else if (key.sym() == FcitxKey_Tab) {
            ch = '\t';
        } else if (key.isSimple()) {
            // ASCII printable incl. digits (needed for VNI 1-9)
            ch = static_cast<uint32_t>(key.sym());
        } else {
            flush(ic, st);
            return;
        }

        const int r = codekey_engine_feed(st->eng, ch);
        switch (static_cast<KR>(r)) {
        case KR::Update:
        case KR::Append:
        case KR::Backspace:
            updateUI(ic, st);
            keyEvent.filterAndAccept();
            break;
        case KR::Commit: {
            // Word separator: commit composition + separator char
            char *committed = codekey_engine_commit(st->eng);
            std::string out = committed ? committed : "";
            if (committed) {
                codekey_string_free(committed);
            }
            // Append separator (space / punct)
            if (ch < 128) {
                out.push_back(static_cast<char>(ch));
            }
            if (!out.empty()) {
                ic->commitString(out);
            }
            clearUI(ic);
            keyEvent.filterAndAccept();
            break;
        }
        case KR::CommitAndPass:
            flush(ic, st);
            // do not filter — key goes to application
            break;
        case KR::Ignored:
        default:
            break;
        }
    }

    void reset(const fcitx::InputMethodEntry &,
               fcitx::InputContextEvent &event) override {
        auto *ic = event.inputContext();
        if (auto *st = ic->propertyFor(&factory_)) {
            if (st->eng) {
                codekey_engine_reset(st->eng);
            }
        }
        clearUI(ic);
    }

    void deactivate(const fcitx::InputMethodEntry &,
                    fcitx::InputContextEvent &event) override {
        auto *ic = event.inputContext();
        auto *st = ic->propertyFor(&factory_);
        if (!st) {
            return;
        }
        // Focus-out: drop preedit; other deactivate: commit
        if (event.type() == fcitx::EventType::InputContextFocusOut) {
            if (st->eng) {
                codekey_engine_reset(st->eng);
            }
            clearUI(ic);
        } else {
            flush(ic, st);
        }
    }

private:
    struct State : fcitx::InputContextProperty {
        ::CodeKeyEngine *eng = nullptr;
        int method = 0; // 0 telex, 1 vni

        ~State() override {
            if (eng) {
                codekey_engine_free(eng);
                eng = nullptr;
            }
        }
    };

    static int methodFromEntry(const fcitx::InputMethodEntry &entry) {
        // uniqueName comes from conf file name / Name field
        const auto &n = entry.uniqueName();
        if (n.find("vni") != std::string::npos ||
            n.find("VNI") != std::string::npos) {
            return 1;
        }
        return 0;
    }

    State *ensure(fcitx::InputContext *ic,
                  const fcitx::InputMethodEntry &entry) {
        auto *st = ic->propertyFor(&factory_);
        const int method = methodFromEntry(entry);
        if (!st->eng || st->method != method) {
            if (st->eng) {
                codekey_engine_free(st->eng);
            }
            st->method = method;
            st->eng = codekey_engine_new(method);
        }
        return st;
    }

    void updateUI(fcitx::InputContext *ic, State *st) {
        char *pre = codekey_engine_preedit(st->eng);
        const std::string text = pre ? pre : "";
        if (pre) {
            codekey_string_free(pre);
        }

        auto &panel = ic->inputPanel();
        panel.reset();
        if (!text.empty()) {
            fcitx::Text preedit;
            preedit.append(text, fcitx::TextFormatFlag::Underline);
            preedit.setCursor(static_cast<int>(fcitx::utf8::length(text)));
            if (ic->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
                panel.setClientPreedit(preedit);
            } else {
                panel.setPreedit(preedit);
            }
        }
        ic->updatePreedit();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    }

    void flush(fcitx::InputContext *ic, State *st) {
        if (st && st->eng) {
            char *committed = codekey_engine_commit(st->eng);
            if (committed && committed[0] != '\0') {
                ic->commitString(committed);
            }
            if (committed) {
                codekey_string_free(committed);
            }
        }
        clearUI(ic);
    }

    static void clearUI(fcitx::InputContext *ic) {
        ic->inputPanel().reset();
        ic->updatePreedit();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    }

    fcitx::Instance *instance_;
    fcitx::FactoryFor<State> factory_{
        [](fcitx::InputContext &) { return new State; }};
};

class CodeKeyFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new CodeKeyEngine(manager->instance());
    }
};

} // namespace

// Standard Fcitx5 addon entry (works across 5.0+).
FCITX_ADDON_FACTORY(CodeKeyFactory)
