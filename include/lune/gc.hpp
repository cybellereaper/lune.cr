#pragma once

#include <memory>
#include <utility>
#include <unordered_set>
#include <vector>

namespace lune {

struct GCObject {
    bool marked{false};
    std::vector<GCObject*> edges;
    virtual ~GCObject() = default;
};

class GarbageCollector {
public:
    template <typename T, typename... Args>
    T* allocate(Args&&... args) {
        auto ptr = std::make_unique<T>(std::forward<Args>(args)...);
        auto* raw = ptr.get();
        objects_.insert(raw);
        storage_.push_back(std::move(ptr));
        return raw;
    }

    void mark(GCObject* root);
    void sweep();
    [[nodiscard]] std::size_t live_objects() const { return objects_.size(); }

private:
    std::unordered_set<GCObject*> objects_;
    std::vector<std::unique_ptr<GCObject>> storage_;
};

} // namespace lune
