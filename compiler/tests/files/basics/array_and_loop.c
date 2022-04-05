int main() {
  int test[5];

  for (int i = 0; i < 5; i++) {
    test[i] = 10;
  }

  test[3] = 0;

  return test[3];
}