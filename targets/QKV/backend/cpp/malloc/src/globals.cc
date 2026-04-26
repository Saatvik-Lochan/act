#include "solver.h"
#include "storage.h"

operations_research::Solver solver("sched_alloc");

std::map<std::string, Storage *> g_storage = {
    {"HBM", new Storage("HBM", {HBM_SIZE}, 0)},
	{"D1", new Storage("D1", {128}, 1)},
	{"D2", new Storage("D2", {64}, 1)}
};
