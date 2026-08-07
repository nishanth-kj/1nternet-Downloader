add_executable(${PROJECT_NAME} main.cpp)

# Later, add linking for necessary libraries (e.g. curl)
target_link_libraries(${PROJECT_NAME} PRIVATE curl cppjson)
