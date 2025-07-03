#include "SandiaDecay.h"
#include <cstddef>
#include <exception>
#include <stdint.h>
#include <string.h>
#include <vector>

using std::size_t;

namespace sdecay {

void std_string_from_cstr(std::string *out, const char *cstr);

void std_string_from_bytes(std::string *out, const char *buffer, size_t size);

const char *std_string_cstr(const std::string *self);
void std_string_bytes(const std::string *self, const char **ptr, size_t *len);

void std_string_destruct(std::string *self);

#define STD_VEC_OPS(name, type)                                                \
    typedef std::vector<type> name##_vec;                                      \
    struct _dummy_##name##_vec {                                               \
        name##_vec inner;                                                      \
    };                                                                         \
                                                                               \
    void std_vector_##name##_new(name##_vec *out);                             \
                                                                               \
    void std_vector_##name##_reserve(name##_vec *self, size_t capacity);       \
                                                                               \
    void std_vector_##name##_push(name##_vec *self, type *item);               \
                                                                               \
    void std_vector_##name##_from_data(type const *data, size_t len,           \
                                       name##_vec *out);                       \
    size_t std_vector_##name##_size(const name##_vec *self);                   \
                                                                               \
    bool std_vector_##name##_empty(name##_vec const *self);                    \
                                                                               \
    type const *std_vector_##name##_ptr(const name##_vec *self);               \
                                                                               \
    type *std_vector_##name##_ptr_mut(name##_vec *self);                       \
                                                                               \
    void std_vector_##name##_destruct(name##_vec *self);

STD_VEC_OPS(char, char);
STD_VEC_OPS(transition, SandiaDecay::Transition);
STD_VEC_OPS(transition_ptr, const SandiaDecay::Transition *);
STD_VEC_OPS(rad_particle, SandiaDecay::RadParticle);
STD_VEC_OPS(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
STD_VEC_OPS(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
STD_VEC_OPS(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
STD_VEC_OPS(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
STD_VEC_OPS(energy_count_pair, SandiaDecay::EnergyCountPair);
STD_VEC_OPS(energy_rate_pair, SandiaDecay::EnergyRatePair);
STD_VEC_OPS(nuclide, SandiaDecay::Nuclide);
STD_VEC_OPS(nuclide_ref, const SandiaDecay::Nuclide *);
STD_VEC_OPS(nuclide_raw_ptr, const SandiaDecay::Nuclide *);
STD_VEC_OPS(element, SandiaDecay::Element);
STD_VEC_OPS(element_raw_ptr, const SandiaDecay::Element *);
STD_VEC_OPS(element_ref, const SandiaDecay::Element *);
typedef std::pair<unsigned short int, float> CoincidencePair;
STD_VEC_OPS(coincidence_pair, CoincidencePair);
STD_VEC_OPS(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
STD_VEC_OPS(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

class Exception {
  public:
    static Exception catch_current();

    static const char *what(Exception const &);

    static void destruct(Exception &);

    // thanks cgpt :melt:
    alignas(std::exception_ptr) uint8_t inner[sizeof(std::exception_ptr)];
};

static_assert(std::is_move_constructible_v<Exception>,
              "Error must be move-constructible");

#define TRY_CALL(name, ret_type, ...)                                          \
    bool try_##name(ret_type *out, Exception *error, ##__VA_ARGS__);

typedef struct {
} Unit;

#define OUT_CALL(name, recvt, rt, ...)                                         \
    void name(rt *out, recvt self, ##__VA_ARGS__);

#define MOVE(name, type) void move_##name(type *dst, type *src);

MOVE(database, SandiaDecay::SandiaDecayDataBase);
MOVE(mixture, SandiaDecay::NuclideMixture);
MOVE(string, std::string);
MOVE(nuclide, SandiaDecay::Nuclide);
MOVE(transition, SandiaDecay::Transition);
MOVE(rad_particle, SandiaDecay::RadParticle);
MOVE(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
MOVE(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
MOVE(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
MOVE(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
MOVE(energy_count_pair, SandiaDecay::EnergyCountPair);
MOVE(energy_rate_pair, SandiaDecay::EnergyRatePair);
MOVE(element, SandiaDecay::Element);
MOVE(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
MOVE(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

#define MOVE_VEC(name, type)                                                   \
    void move_##name##_vec(std::vector<type> *dst, std::vector<type> *src);

MOVE_VEC(char, char);
MOVE_VEC(transition, SandiaDecay::Transition);
MOVE_VEC(transition_ptr, const SandiaDecay::Transition *);
MOVE_VEC(rad_particle, SandiaDecay::RadParticle);
MOVE_VEC(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
MOVE_VEC(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
MOVE_VEC(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
MOVE_VEC(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
MOVE_VEC(energy_count_pair, SandiaDecay::EnergyCountPair);
MOVE_VEC(energy_rate_pair, SandiaDecay::EnergyRatePair);
MOVE_VEC(nuclide, SandiaDecay::Nuclide);
MOVE_VEC(nuclide_ref, const SandiaDecay::Nuclide *);
MOVE_VEC(nuclide_raw_ptr, const SandiaDecay::Nuclide *);
MOVE_VEC(element, SandiaDecay::Element);
MOVE_VEC(element_raw_ptr, const SandiaDecay::Element *);
MOVE_VEC(element_ref, const SandiaDecay::Element *);
MOVE_VEC(coincidence_pair, CoincidencePair);
MOVE_VEC(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
MOVE_VEC(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

namespace database {

// bool try_init_database1(SandiaDecay::SandiaDecayDataBase *database,
//                         const char *path_cstr, Error *error);

// bool try_init_database_bytes(SandiaDecay::SandiaDecayDataBase *database,
//                              const char *bytes_ptr, size_t bytes_len,
//                              Error *error);

TRY_CALL(init_database, Unit, SandiaDecay::SandiaDecayDataBase *database,
         std::string const &path);

TRY_CALL(init_database_bytes, Unit, SandiaDecay::SandiaDecayDataBase *database,
         std::vector<char> &data);

void decay_single(std::vector<SandiaDecay::NuclideActivityPair> *out,
                  const SandiaDecay::Nuclide *parent, double original_activity,
                  double time_in_seconds);

void decay_atoms(std::vector<SandiaDecay::NuclideActivityPair> *out,
                 const std::vector<SandiaDecay::NuclideNumAtomsPair> &parents,
                 double time);

void decay_activities(
    std::vector<SandiaDecay::NuclideActivityPair> *out,
    const std::vector<SandiaDecay::NuclideActivityPair> &parents, double time);

void decay_activities_assign(
    std::vector<SandiaDecay::NuclideActivityPair> &parents, double time);

void evolution_single(std::vector<SandiaDecay::NuclideTimeEvolution> *out,
                      const SandiaDecay::Nuclide *parent,
                      double original_activity);

void evolution_atoms(
    std::vector<SandiaDecay::NuclideTimeEvolution> *out,
    const std::vector<SandiaDecay::NuclideNumAtomsPair> &parents);

void evolution_activities(
    std::vector<SandiaDecay::NuclideTimeEvolution> *out,
    const std::vector<SandiaDecay::NuclideActivityPair> &parents);

} // namespace database

namespace nuclide {

// void descendants1(const SandiaDecay::Nuclide *nuclide,
//                   std::vector<const SandiaDecay::Nuclide *> *out);

// void forebearers(const SandiaDecay::Nuclide *nuclide,
//                  std::vector<const SandiaDecay::Nuclide *> *out);

// void human_str_summary(const SandiaDecay::Nuclide *nuclide, std::string
// *out);

OUT_CALL(descendants, const SandiaDecay::Nuclide *,
         std::vector<const SandiaDecay::Nuclide *>);
OUT_CALL(forebearers, const SandiaDecay::Nuclide *,
         std::vector<const SandiaDecay::Nuclide *>);
OUT_CALL(human_str_summary, const SandiaDecay::Nuclide *, std::string);

} // namespace nuclide

namespace nuclide_mixture {

// bool try_activity_nuclide(const SandiaDecay::NuclideMixture *self, double
// time,
//                           const SandiaDecay::Nuclide *nuclide, double
//                           *result, Error *error);

// void activities(const SandiaDecay::NuclideMixture *mixture, double time,
//                 std::vector<SandiaDecay::NuclideActivityPair> *out);

// void gammas(const SandiaDecay::NuclideMixture *mixture, double time,
//             SandiaDecay::NuclideMixture::HowToOrder ordering,
//             bool includeAnnihillations,
//             std::vector<SandiaDecay::EnergyRatePair> *out);

// void xrays(const SandiaDecay::NuclideMixture *mixture, double time,
//            SandiaDecay::NuclideMixture::HowToOrder ordering,
//            std::vector<SandiaDecay::EnergyRatePair> *out);
//
// void photons(const SandiaDecay::NuclideMixture *mixture, double time,
//              SandiaDecay::NuclideMixture::HowToOrder ordering,
//              std::vector<SandiaDecay::EnergyRatePair> *out);
//
// void decayPhotonsInInterval(const SandiaDecay::NuclideMixture *mixture,
//                             double initial_age, double measurement_duration,
//                             SandiaDecay::NuclideMixture::HowToOrder ordering,
//                             size_t num_timeslices,
//                             std::vector<SandiaDecay::EnergyCountPair> *out);
//
// void numAtoms(const SandiaDecay::NuclideMixture *mixture, double time,
//               std::vector<SandiaDecay::NuclideNumAtomsPair> *out);

// OUT_CALL(decayedToNuclidesEvolutions, const SandiaDecay::NuclideMixture *,
//          std::vector<SandiaDecay::NuclideTimeEvolution>);

OUT_CALL(activity, const SandiaDecay::NuclideMixture *,
         std::vector<SandiaDecay::NuclideActivityPair>, double time);

TRY_CALL(gammas, std::vector<SandiaDecay::EnergyRatePair>,
         const SandiaDecay::NuclideMixture *mixture, double time,
         SandiaDecay::NuclideMixture::HowToOrder ordering,
         bool include_annihillations);

TRY_CALL(alphas, std::vector<SandiaDecay::EnergyRatePair>,
         const SandiaDecay::NuclideMixture *mixture, double time,
         SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL(betas, std::vector<SandiaDecay::EnergyRatePair>,
         const SandiaDecay::NuclideMixture *mixture, double time,
         SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL(betaPlusses, std::vector<SandiaDecay::EnergyRatePair>,
         const SandiaDecay::NuclideMixture *mixture, double time,
         SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL(decayParticle, std::vector<SandiaDecay::EnergyRatePair>,
         const SandiaDecay::NuclideMixture *mixture, double time,
         SandiaDecay::ProductType type,
         SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL(decayParticlesInInterval, std::vector<SandiaDecay::EnergyCountPair>,
         const SandiaDecay::NuclideMixture *mixture, double initial_age,
         double interval_duration, SandiaDecay::ProductType type,
         SandiaDecay::NuclideMixture::HowToOrder sort_type,
         size_t characteristic_time_slices);

TRY_CALL(decayPhotonsInInterval, std::vector<SandiaDecay::EnergyCountPair>,
         const SandiaDecay::NuclideMixture *mixture, double initial_age,
         double interval_duration,
         SandiaDecay::NuclideMixture::HowToOrder sort_type,
         size_t characteristic_time_slices);

TRY_CALL(decayGammasInInterval, std::vector<SandiaDecay::EnergyCountPair>,
         const SandiaDecay::NuclideMixture *mixture, double initial_age,
         double interval_duration, bool includeAnnihilation,
         SandiaDecay::NuclideMixture::HowToOrder sort_type,
         size_t characteristic_time_slices);

TRY_CALL(xrays, std::vector<SandiaDecay::EnergyRatePair>,
         const SandiaDecay::NuclideMixture *mixture, double time,
         SandiaDecay::NuclideMixture::HowToOrder ordering);

TRY_CALL(photons, std::vector<SandiaDecay::EnergyRatePair>,
         const SandiaDecay::NuclideMixture *mixture, double time,
         SandiaDecay::NuclideMixture::HowToOrder ordering);

OUT_CALL(numAtoms, const SandiaDecay::NuclideMixture *,
         std::vector<SandiaDecay::NuclideNumAtomsPair>, double time);

OUT_CALL(info, const SandiaDecay::NuclideMixture *, std::string, double time);

TRY_CALL(activity_nuclide, double, SandiaDecay::NuclideMixture const *mixture,
         double time, SandiaDecay::Nuclide const *nuclide);

TRY_CALL(activity_symbol, double, SandiaDecay::NuclideMixture const *mixture,
         double time, std::string const &symbol);

TRY_CALL(activity_num, double, SandiaDecay::NuclideMixture const *mixture,
         double time, int z, int atomic_mass, int iso);

TRY_CALL(atoms_nuclide, double, SandiaDecay::NuclideMixture const *mixture,
         double time, SandiaDecay::Nuclide const *nuclide);

TRY_CALL(atoms_symbol, double, SandiaDecay::NuclideMixture const *mixture,
         double time, std::string const &symbol);

TRY_CALL(atoms_num, double, SandiaDecay::NuclideMixture const *mixture,
         double time, int z, int atomic_mass, int iso);

TRY_CALL(addAgedNuclideByActivity, Unit, SandiaDecay::NuclideMixture *mixture,
         const SandiaDecay::Nuclide *nuclide, double activity,
         double age_in_seconds);

TRY_CALL(addAgedNuclideByNumAtoms, Unit, SandiaDecay::NuclideMixture *mixture,
         const SandiaDecay::Nuclide *nuclide, double number_atoms,
         double age_in_seconds);

} // namespace nuclide_mixture

namespace transition {

OUT_CALL(human_str_summary, const SandiaDecay::Transition *, std::string);

} // namespace transition

namespace rad_particle {

OUT_CALL(human_str_summary, const SandiaDecay::RadParticle *, std::string);

}

namespace layout {

#define LAYOUT(name, type)                                                     \
    namespace name {                                                           \
    extern const size_t size;                                                  \
    extern const size_t align;                                                 \
    }

LAYOUT(std_string, std::string);
LAYOUT(database, SandiaDecay::SandiaDecayDataBase);
LAYOUT(mixture, SandiaDecay::NuclideMixture);
LAYOUT(string, std::string);
LAYOUT(nuclide, SandiaDecay::Nuclide);
LAYOUT(transition, SandiaDecay::Transition);
LAYOUT(rad_particle, SandiaDecay::RadParticle);
LAYOUT(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
LAYOUT(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
LAYOUT(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
LAYOUT(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
LAYOUT(energy_count_pair, SandiaDecay::EnergyCountPair);
LAYOUT(energy_rate_pair, SandiaDecay::EnergyRatePair);
LAYOUT(element, SandiaDecay::Element);
LAYOUT(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
LAYOUT(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

#define LAYOUT_VEC(name, type)                                                 \
    namespace name##_vec {                                                     \
        extern const size_t size;                                              \
        extern const size_t align;                                             \
    }

LAYOUT_VEC(char, char);
LAYOUT_VEC(transition, SandiaDecay::Transition);
LAYOUT_VEC(transition_ptr, const SandiaDecay::Transition *);
LAYOUT_VEC(rad_particle, SandiaDecay::RadParticle);
LAYOUT_VEC(nuclide_abundance_pair, SandiaDecay::NuclideAbundancePair);
LAYOUT_VEC(nuclide_activity_pair, SandiaDecay::NuclideActivityPair);
LAYOUT_VEC(nuclide_num_atoms_pair, SandiaDecay::NuclideNumAtomsPair);
LAYOUT_VEC(energy_intensity_pair, SandiaDecay::EnergyIntensityPair);
LAYOUT_VEC(energy_count_pair, SandiaDecay::EnergyCountPair);
LAYOUT_VEC(energy_rate_pair, SandiaDecay::EnergyRatePair);
LAYOUT_VEC(nuclide, SandiaDecay::Nuclide);
LAYOUT_VEC(nuclide_ref, const SandiaDecay::Nuclide *);
LAYOUT_VEC(nuclide_raw_ptr, const SandiaDecay::Nuclide *);
LAYOUT_VEC(element, SandiaDecay::Element);
LAYOUT_VEC(element_raw_ptr, const SandiaDecay::Element *);
LAYOUT_VEC(element_ref, const SandiaDecay::Element *);
LAYOUT_VEC(coincidence_pair, CoincidencePair);
LAYOUT_VEC(time_evolution_term, SandiaDecay::TimeEvolutionTerm);
LAYOUT_VEC(nuclide_time_evolution, SandiaDecay::NuclideTimeEvolution);

} // namespace layout

} // namespace sdecay
