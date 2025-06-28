#include "wrapper.hpp"

#include "SandiaDecay.h"
#include <cstdlib>
#include <exception>
#include <string.h>
#include <vector>

// cred: cGPT
// prompt: alike "please give me Rust core::ptr::write, but in C++"
template <typename T> inline void write(T *dst, T src) {
    new (dst) T(std::move(src));
}

// cred: cGPT
// prompt: alike "please give me Rust core::ptr::read, but in C++"
template <typename T> void move_from_to(T *dst, T *src) {
    static_assert(std::is_move_constructible_v<T>,
                  "T must be move constructible");
    static_assert(std::is_destructible_v<T>, "T must be destructible");

    // I still have no idea what this warning means exactly
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated-copy"
    ::new (static_cast<void *>(dst)) T(std::move(*src));
#pragma GCC diagnostic pop
    // actually destruct value at `src`
    src->~T();
}

namespace sdecay {

void std_string_from_cstr(std::string *out, const char *cstr) {
    auto res = std::string(cstr);
    write(out, res);
}

void std_string_from_bytes(std::string *out, const char *buffer,
                           std::size_t size) {
    auto res = std::string(buffer, size);
    write(out, res);
}

const char *std_string_cstr(const std::string *self) { return self->c_str(); }
void std_string_bytes(const std::string *self, const char **ptr, size_t *len) {
    *ptr = self->c_str();
    *len = self->length();
}

void std_string_destruct(std::string *self) { self->~basic_string(); }

#define STD_VEC_OPS_DEF(name, typ)                                             \
    void std_vector_##name##_new(name##_vec *out) {                            \
        write(out, name##_vec());                                              \
    }                                                                          \
                                                                               \
    void std_vector_##name##_reserve(name##_vec *self, size_t capacity) {      \
        self->reserve(capacity);                                               \
    }                                                                          \
                                                                               \
    void std_vector_##name##_push(name##_vec *self, typ *item) {               \
        self->push_back(std::move(*item));                                     \
    }                                                                          \
    void std_vector_##name##_from_data(typ const *data, size_t len,            \
                                       name##_vec *out) {                      \
        auto res = std::vector(data, data + len);                              \
        write(out, res);                                                       \
    }                                                                          \
    size_t std_vector_##name##_size(const name##_vec *self) {                  \
        return self->size();                                                   \
    }                                                                          \
                                                                               \
    bool std_vector_##name##_empty(name##_vec const *self) {                   \
        return self->empty();                                                  \
    }                                                                          \
                                                                               \
    typ const *std_vector_##name##_ptr(const name##_vec *self) {               \
        return self->data();                                                   \
    }                                                                          \
                                                                               \
    typ *std_vector_##name##_ptr_mut(name##_vec *self) {                       \
        return self->data();                                                   \
    }                                                                          \
                                                                               \
    void std_vector_##name##_destruct(name##_vec *self) { self->~vector(); }

STD_VEC_OPS_DEF(char, char);
STD_VEC_OPS_DEF(transition, SandiaDecay::Transition);
STD_VEC_OPS_DEF(transition_ptr, const SandiaDecay::Transition *);
STD_VEC_OPS_DEF(rad_particle, SandiaDecay::RadParticle);
STD_VEC_OPS_DEF(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
STD_VEC_OPS_DEF(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
STD_VEC_OPS_DEF(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
STD_VEC_OPS_DEF(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
STD_VEC_OPS_DEF(energy_count_pair, SandiaDecay::EnergyCountPair);
STD_VEC_OPS_DEF(energy_rate_pair, SandiaDecay::EnergyRatePair);
STD_VEC_OPS_DEF(nuclide, SandiaDecay::Nuclide);
STD_VEC_OPS_DEF(nuclide_ref, const SandiaDecay::Nuclide *);
STD_VEC_OPS_DEF(nuclide_raw_ptr, const SandiaDecay::Nuclide *);
STD_VEC_OPS_DEF(element, SandiaDecay::Element);
STD_VEC_OPS_DEF(element_ref, const SandiaDecay::Element *);
STD_VEC_OPS_DEF(element_raw_ptr, const SandiaDecay::Element *);
STD_VEC_OPS_DEF(coincidence_pair, CoincidencePair);
STD_VEC_OPS_DEF(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
STD_VEC_OPS_DEF(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

Exception Exception::catch_current() {
    Exception e;
    new (e.inner) std::exception_ptr(std::current_exception());
    return e;
}

const char *Exception::what(Exception const &ex) {
    try {
        auto ptr = reinterpret_cast<std::exception_ptr const *>(ex.inner);
        std::rethrow_exception(*ptr);
    } catch (const std::exception &ex) {
        return ex.what();
    }
}

void Exception::destruct(Exception &ex) {
    auto *ptr = reinterpret_cast<std::exception_ptr *>(ex.inner);
    ptr->~exception_ptr();
}

#define TRY_CALL_DEF(name, ret_type, call, ...)                                \
    bool try_##name(ret_type *out, Exception *error, ##__VA_ARGS__) {          \
        try {                                                                  \
            ret_type res = call;                                               \
            write(out, res);                                                   \
            return true;                                                       \
        } catch (...) {                                                        \
            auto ex = Exception::catch_current();                              \
            write(error, ex);                                                  \
            return false;                                                      \
        }                                                                      \
    }

#define OUT_CALL_DEF(name, recvt, rt, cargs, ...)                              \
    void name(rt *out, recvt self, ##__VA_ARGS__) {                            \
        auto res = self->name cargs;                                           \
        write(out, res);                                                       \
    }

#define MOVE_DEF(name, typ)                                                    \
    void move_##name(typ *dst, typ *src) { move_from_to(dst, src); }

MOVE_DEF(database, SandiaDecay::SandiaDecayDataBase);
MOVE_DEF(mixture, SandiaDecay::NuclideMixture);
MOVE_DEF(string, std::string);
MOVE_DEF(nuclide, SandiaDecay::Nuclide);
MOVE_DEF(transition, SandiaDecay::Transition);
MOVE_DEF(rad_particle, SandiaDecay::RadParticle);
MOVE_DEF(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
MOVE_DEF(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
MOVE_DEF(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
MOVE_DEF(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
MOVE_DEF(energy_count_pair, SandiaDecay::EnergyCountPair);
MOVE_DEF(energy_rate_pair, SandiaDecay::EnergyRatePair);
MOVE_DEF(element, SandiaDecay::Element);
MOVE_DEF(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
MOVE_DEF(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

#define MOVE_VEC_DEF(name, typ)                                                \
    void move_##name##_vec(std::vector<typ> *dst, std::vector<typ> *src) {     \
        move_from_to(dst, src);                                                \
    }

MOVE_VEC_DEF(char, char);
MOVE_VEC_DEF(transition, SandiaDecay::Transition);
MOVE_VEC_DEF(transition_ptr, const SandiaDecay::Transition *);
MOVE_VEC_DEF(rad_particle, SandiaDecay::RadParticle);
MOVE_VEC_DEF(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
MOVE_VEC_DEF(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
MOVE_VEC_DEF(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
MOVE_VEC_DEF(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
MOVE_VEC_DEF(energy_count_pair, SandiaDecay::EnergyCountPair);
MOVE_VEC_DEF(energy_rate_pair, SandiaDecay::EnergyRatePair);
MOVE_VEC_DEF(nuclide, SandiaDecay::Nuclide);
MOVE_VEC_DEF(nuclide_ref, const SandiaDecay::Nuclide *);
MOVE_VEC_DEF(nuclide_raw_ptr, const SandiaDecay::Nuclide *);
MOVE_VEC_DEF(element, SandiaDecay::Element);
MOVE_VEC_DEF(element_raw_ptr, const SandiaDecay::Element *);
MOVE_VEC_DEF(element_ref, const SandiaDecay::Element *);
MOVE_VEC_DEF(coincidence_pair, CoincidencePair);
MOVE_VEC_DEF(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
MOVE_VEC_DEF(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

namespace database {

// bool try_init_database1(SandiaDecay::SandiaDecayDataBase *database,
//                         const char *path_cstr, Error *error) {
//     std::string path(path_cstr);
//     try {
//         database->initialize(path);
//         return true;
//     } catch (std::exception &ex) {
//         auto err = Error(ex);
//         ::new (static_cast<void *>(error)) Error(std::move(err));
//         return false;
//     }
// }

// bool try_init_database_bytes(SandiaDecay::SandiaDecayDataBase *database,
//                              const char *bytes_ptr, size_t bytes_len,
//                              Error *error) {
//     std::vector<char> bytes(bytes_ptr, bytes_ptr + bytes_len);
//     bytes.push_back(0);
//     try {
//         database->initialize(bytes);
//         return true;
//     } catch (std::exception &ex) {
//         auto err = Error(ex);
//         ::new (static_cast<void *>(error)) Error(std::move(err));
//         return false;
//     }
// }

TRY_CALL_DEF(init_database, Unit, ([database, path] {
                 database->initialize(path);
                 return Unit();
             }()),
             SandiaDecay::SandiaDecayDataBase *database,
             std::string const &path);

TRY_CALL_DEF(init_database_bytes, Unit, ([database, data]() mutable {
                 database->initialize(data);
                 return Unit();
             }()),
             SandiaDecay::SandiaDecayDataBase *database,
             std::vector<char> &data);

void decay_single(const SandiaDecay::Nuclide *parent,
                  std::vector<SandiaDecay::NuclideActivityPair> *out,
                  double original_activity, double time_in_seconds) {
    auto res = SandiaDecay::SandiaDecayDataBase::decay(
        parent, original_activity, time_in_seconds);
    write(out, res);
}

void decay_atoms(std::vector<SandiaDecay::NuclideActivityPair> *out,
                 const std::vector<SandiaDecay::NuclideNumAtomsPair> &parents,
                 double time) {
    auto res = SandiaDecay::SandiaDecayDataBase::decay(parents, time);
    write(out, res);
}

void decay_activities(
    std::vector<SandiaDecay::NuclideActivityPair> *out,
    const std::vector<SandiaDecay::NuclideActivityPair> &parents, double time) {
    auto res = SandiaDecay::SandiaDecayDataBase::decay(parents, time);
    write(out, res);
}

void decay_activities_assign(
    std::vector<SandiaDecay::NuclideActivityPair> &parents, double time) {
    auto res = SandiaDecay::SandiaDecayDataBase::decay(parents, time);
    parents = res;
}

void evolution_single(std::vector<SandiaDecay::NuclideTimeEvolution> *out,
                      const SandiaDecay::Nuclide *parent,
                      double original_activity) {
    auto res = SandiaDecay::SandiaDecayDataBase::getTimeEvolution(
        parent, original_activity);
    write(out, res);
}

void evolution_atoms(
    std::vector<SandiaDecay::NuclideTimeEvolution> *out,
    const std::vector<SandiaDecay::NuclideNumAtomsPair> &parents) {
    auto res = SandiaDecay::SandiaDecayDataBase::getTimeEvolution(parents);
    write(out, res);
}

void evolution_activities(
    std::vector<SandiaDecay::NuclideTimeEvolution> *out,
    const std::vector<SandiaDecay::NuclideActivityPair> &parents) {
    auto res = SandiaDecay::SandiaDecayDataBase::getTimeEvolution(parents);
    write(out, res);
}

} // namespace database

const SandiaDecay::Nuclide *
nuclide_by_name(const SandiaDecay::SandiaDecayDataBase *self,
                const char *label_ctr) {
    auto label = std::string(label_ctr);
    return self->nuclide(label);
}

namespace nuclide {

// void descendants1(const SandiaDecay::Nuclide *nuclide,
//                   std::vector<const SandiaDecay::Nuclide *> *out) {
//     auto res = nuclide->descendants();
//     write(out, res);
// }

// void forebearers(const SandiaDecay::Nuclide *nuclide,
//                  std::vector<const SandiaDecay::Nuclide *> *out) {
//     auto res = nuclide->forebearers();
//     write(out, res);
// }

// (this method belongs to a different type, for some reason is non-standand)
void human_str_summary(std::string *out, const SandiaDecay::Nuclide *nuclide) {
    auto res = SandiaDecay::human_str_summary(*nuclide);
    write(out, res);
}

OUT_CALL_DEF(descendants, const SandiaDecay::Nuclide *,
             std::vector<const SandiaDecay::Nuclide *>, ());
OUT_CALL_DEF(forebearers, const SandiaDecay::Nuclide *,
             std::vector<const SandiaDecay::Nuclide *>, ());
// OUT_CALL_DEF(human_str_summary, const SandiaDecay::Nuclide *, std::string);

} // namespace nuclide

namespace nuclide_mixture {

// void activities(const SandiaDecay::NuclideMixture *mixture, double time,
//                 std::vector<SandiaDecay::NuclideActivityPair> *out) {
//     auto res = mixture->activity(time);
//     write(out, res);
// }
//
// void gammas(const SandiaDecay::NuclideMixture *mixture, double time,
//             SandiaDecay::NuclideMixture::HowToOrder ordering,
//             bool includeAnnihillations,
//             std::vector<SandiaDecay::EnergyRatePair> *out) {
//     auto res = mixture->gammas(time, ordering, includeAnnihillations);
//     write(out, res);
// }

// void xrays(const SandiaDecay::NuclideMixture *mixture, double time,
//            SandiaDecay::NuclideMixture::HowToOrder ordering,
//            std::vector<SandiaDecay::EnergyRatePair> *out) {
//     auto res = mixture->xrays(time, ordering);
//     write(out, res);
// }
//
// void photons(const SandiaDecay::NuclideMixture *mixture, double time,
//              SandiaDecay::NuclideMixture::HowToOrder ordering,
//              std::vector<SandiaDecay::EnergyRatePair> *out) {
//     auto res = mixture->photons(time, ordering);
//     write(out, res);
// }
//
// void decayPhotonsInInterval(const SandiaDecay::NuclideMixture *mixture,
//                             double initial_age, double measurement_duration,
//                             SandiaDecay::NuclideMixture::HowToOrder ordering,
//                             size_t num_timeslices,
//                             std::vector<SandiaDecay::EnergyCountPair> *out) {
//     auto res = mixture->decayPhotonsInInterval(
//         initial_age, measurement_duration, ordering, num_timeslices);
//     write(out, res);
// }
//
// void numAtoms(const SandiaDecay::NuclideMixture *mixture, double time,
//               std::vector<SandiaDecay::NuclideNumAtomsPair> *out) {
//     auto res = mixture->numAtoms(time);
//     write(out, res);
// }

// bool try_activity_nuclide(const SandiaDecay::NuclideMixture *self, double
// time,
//                           const SandiaDecay::Nuclide *nuclide, double
//                           *result, Error *error) {
//     try {
//         *result = self->activity(time, nuclide);
//         return true;
//     } catch (std::exception &ex) {
//         auto err = Error(ex);
//         ::new (static_cast<void *>(error)) Error(std::move(err));
//         return false;
//     }
// }

// OUT_CALL_DEF(decayedToNuclidesEvolutions, const SandiaDecay::NuclideMixture
// *,
//              std::vector<SandiaDecay::NuclideTimeEvolution>, ());

OUT_CALL_DEF(activity, const SandiaDecay::NuclideMixture *,
             std::vector<SandiaDecay::NuclideActivityPair>, (time),
             double time);

TRY_CALL_DEF(gammas, std::vector<SandiaDecay::EnergyRatePair>,
             mixture->gammas(time, ordering, include_annihillations),
             const SandiaDecay::NuclideMixture *mixture, double time,
             SandiaDecay::NuclideMixture::HowToOrder ordering,
             bool include_annihillations);

TRY_CALL_DEF(alphas, std::vector<SandiaDecay::EnergyRatePair>,
             mixture->alphas(time, ordering),
             const SandiaDecay::NuclideMixture *mixture, double time,
             SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL_DEF(betas, std::vector<SandiaDecay::EnergyRatePair>,
             mixture->betas(time, ordering),
             const SandiaDecay::NuclideMixture *mixture, double time,
             SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL_DEF(betaPlusses, std::vector<SandiaDecay::EnergyRatePair>,
             mixture->betaPlusses(time, ordering),
             const SandiaDecay::NuclideMixture *mixture, double time,
             SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL_DEF(decayParticle, std::vector<SandiaDecay::EnergyRatePair>,
             mixture->decayParticle(time, type, ordering),
             const SandiaDecay::NuclideMixture *mixture, double time,
             SandiaDecay::ProductType type,
             SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL_DEF(decayParticlesInInterval,
             std::vector<SandiaDecay::EnergyCountPair>,
             mixture->decayParticlesInInterval(initial_age, interval_duration,
                                               type, sort_type,
                                               characteristic_time_slices),
             const SandiaDecay::NuclideMixture *mixture, double initial_age,
             double interval_duration, SandiaDecay::ProductType type,
             SandiaDecay::NuclideMixture::HowToOrder sort_type,
             size_t characteristic_time_slices);

TRY_CALL_DEF(decayPhotonsInInterval, std::vector<SandiaDecay::EnergyCountPair>,
             mixture->decayPhotonsInInterval(initial_age, interval_duration,
                                             sort_type,
                                             characteristic_time_slices),
             const SandiaDecay::NuclideMixture *mixture, double initial_age,
             double interval_duration,
             SandiaDecay::NuclideMixture::HowToOrder sort_type,
             size_t characteristic_time_slices);

TRY_CALL_DEF(decayGammasInInterval, std::vector<SandiaDecay::EnergyCountPair>,
             mixture->decayGammasInInterval(initial_age, interval_duration,
                                            includeAnnihilation, sort_type,
                                            characteristic_time_slices),
             const SandiaDecay::NuclideMixture *mixture, double initial_age,
             double interval_duration, bool includeAnnihilation,
             SandiaDecay::NuclideMixture::HowToOrder sort_type,
             size_t characteristic_time_slices);

TRY_CALL_DEF(xrays, std::vector<SandiaDecay::EnergyRatePair>,
             mixture->xrays(time, ordering),
             const SandiaDecay::NuclideMixture *mixture, double time,
             SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL_DEF(photons, std::vector<SandiaDecay::EnergyRatePair>,
             mixture->photons(time, ordering),
             const SandiaDecay::NuclideMixture *mixture, double time,
             SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL_DEF(activity_nuclide, double, mixture->activity(time, nuclide),
             SandiaDecay::NuclideMixture const *mixture, double time,
             SandiaDecay::Nuclide const *nuclide);

TRY_CALL_DEF(activity_symbol, double, mixture->activity(time, symbol),
             SandiaDecay::NuclideMixture const *mixture, double time,
             std::string const &symbol);

TRY_CALL_DEF(activity_num, double, mixture->activity(time, z, atomic_mass, iso),
             SandiaDecay::NuclideMixture const *mixture, double time, int z,
             int atomic_mass, int iso);

TRY_CALL_DEF(atoms_nuclide, double, mixture->numAtoms(time, nuclide),
             SandiaDecay::NuclideMixture const *mixture, double time,
             SandiaDecay::Nuclide const *nuclide);

TRY_CALL_DEF(atoms_symbol, double, mixture->numAtoms(time, symbol),
             SandiaDecay::NuclideMixture const *mixture, double time,
             std::string const &symbol);

OUT_CALL_DEF(numAtoms, SandiaDecay::NuclideMixture const *,
             std::vector<SandiaDecay::NuclideNumAtomsPair>, (time),
             double time);

OUT_CALL_DEF(info, const SandiaDecay::NuclideMixture *, std::string, (time),
             double time);

TRY_CALL_DEF(addAgedNuclideByActivity, Unit,
             ([mixture, nuclide, activity, age_in_seconds] {
                 mixture->addAgedNuclideByActivity(nuclide, activity,
                                                   age_in_seconds);
                 return Unit();
             }()),
             SandiaDecay::NuclideMixture *mixture,
             const SandiaDecay::Nuclide *nuclide, double activity,
             double age_in_seconds);

TRY_CALL_DEF(addAgedNuclideByNumAtoms, Unit,
             ([mixture, nuclide, number_atoms, age_in_seconds] {
                 mixture->addAgedNuclideByNumAtoms(nuclide, number_atoms,
                                                   age_in_seconds);
                 return Unit();
             }()),
             SandiaDecay::NuclideMixture *mixture,
             const SandiaDecay::Nuclide *nuclide, double number_atoms,
             double age_in_seconds);

} // namespace nuclide_mixture

namespace transition {

// (this method is non-standand)
void human_str_summary(const SandiaDecay::Transition *trans, std::string *out) {
    auto res = SandiaDecay::human_str_summary(*trans);
    write(out, res);
}

} // namespace transition

namespace rad_particle {

// (this method is non-standand)
void human_str_summary(const SandiaDecay::RadParticle *rad_particle,
                       std::string *out) {
    auto res = SandiaDecay::human_str_summary(*rad_particle);
    write(out, res);
}

} // namespace rad_particle

namespace layout {

#define LAYOUT_DEF(name, typ)                                                  \
    namespace name {                                                           \
    const size_t size = sizeof(typ);                                           \
    const size_t align = alignof(typ);                                         \
    }

LAYOUT_DEF(std_string, std::string);
LAYOUT_DEF(database, SandiaDecay::SandiaDecayDataBase);
LAYOUT_DEF(mixture, SandiaDecay::NuclideMixture);
LAYOUT_DEF(string, std::string);
LAYOUT_DEF(nuclide, SandiaDecay::Nuclide);
LAYOUT_DEF(transition, SandiaDecay::Transition);
LAYOUT_DEF(rad_particle, SandiaDecay::RadParticle);
LAYOUT_DEF(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
LAYOUT_DEF(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
LAYOUT_DEF(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
LAYOUT_DEF(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
LAYOUT_DEF(energy_count_pair, SandiaDecay::EnergyCountPair);
LAYOUT_DEF(energy_rate_pair, SandiaDecay::EnergyRatePair);
LAYOUT_DEF(element, SandiaDecay::Element);
LAYOUT_DEF(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
LAYOUT_DEF(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

#define LAYOUT_VEC_DEF(name, typ)                                              \
    namespace name##_vec {                                                     \
        const size_t size = sizeof(std::vector<typ>);                          \
        const size_t align = alignof(std::vector<typ>);                        \
    }

LAYOUT_VEC_DEF(char, char);
LAYOUT_VEC_DEF(transition, SandiaDecay::Transition);
LAYOUT_VEC_DEF(transition_ptr, const SandiaDecay::Transition *);
LAYOUT_VEC_DEF(rad_particle, SandiaDecay::RadParticle);
LAYOUT_VEC_DEF(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
LAYOUT_VEC_DEF(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
LAYOUT_VEC_DEF(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
LAYOUT_VEC_DEF(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
LAYOUT_VEC_DEF(energy_count_pair, SandiaDecay::EnergyCountPair);
LAYOUT_VEC_DEF(energy_rate_pair, SandiaDecay::EnergyRatePair);
LAYOUT_VEC_DEF(nuclide, SandiaDecay::Nuclide);
LAYOUT_VEC_DEF(nuclide_ref, const SandiaDecay::Nuclide *);
LAYOUT_VEC_DEF(nuclide_raw_ptr, const SandiaDecay::Nuclide *);
LAYOUT_VEC_DEF(element, SandiaDecay::Element);
LAYOUT_VEC_DEF(element_raw_ptr, const SandiaDecay::Element *);
LAYOUT_VEC_DEF(element_ref, const SandiaDecay::Element *);
LAYOUT_VEC_DEF(coincidence_pair, CoincidencePair);
LAYOUT_VEC_DEF(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
LAYOUT_VEC_DEF(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

} // namespace layout

} // namespace sdecay
