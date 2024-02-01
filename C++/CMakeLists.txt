cmake_minimum_required(VERSION 3.5)
cmake_policy(SET CMP0074 NEW)

project(lock_free_stack LANGUAGES CXX)
set(PROJECT_NAME lock_free_stack)
set (CMAKE_CXX_STANDARD 17)

set(SOURCES_DIR ${CMAKE_SOURCE_DIR})
add_executable(${PROJECT_NAME}
			   ${SOURCES_DIR}/lock_free_stack.h
			   ${SOURCES_DIR}/stack_example.cpp
)
			   
target_include_directories(${PROJECT_NAME} PRIVATE ${SOURCES_DIR})
