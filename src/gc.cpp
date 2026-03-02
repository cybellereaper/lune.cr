#include "lune/gc.hpp"

namespace lune {

void GarbageCollector::mark(GCObject* root) {
    if (!root || root->marked) {
        return;
    }
    root->marked = true;
    for (auto* edge : root->edges) {
        mark(edge);
    }
}

void GarbageCollector::sweep() {
    for (auto it = storage_.begin(); it != storage_.end();) {
        auto* raw = it->get();
        if (!raw->marked) {
            objects_.erase(raw);
            it = storage_.erase(it);
            continue;
        }
        raw->marked = false;
        ++it;
    }
}

} // namespace lune
