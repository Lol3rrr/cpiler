int fact(int p) {
  if (p == 1) {
    return 1;
  }

  return p * fact(p - 1);
}

int main() {
  return fact(3);
}