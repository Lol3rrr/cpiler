int main() {
  int result = 100;

  for (int i = 0; i < 10; i++) {
    for (int j = 0; j < 10; j++) {
      result -= 1;
    }
  }

  return result;
}