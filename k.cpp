#include <cstdint>
#include <iostream>
#include <windows.h>

int main() {
  uint32_t buffer_size = GetCurrentDirectoryA(0, nullptr);

  char *buffer = static_cast<char *>(malloc(buffer_size));

  uint32_t r = GetCurrentDirectoryA(buffer_size, buffer);

  if (r == 0) {
    std::cout << "Error\n";
    return 1;
  } else if (r == (buffer_size + 1)) {
    std::cout << "wrote full\n";
  } else {
    std::cout << "something else\n";
  }

  std::cout << std::string(buffer) << '\n';

  while (true) {
  }
}
