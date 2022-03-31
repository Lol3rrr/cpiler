int main() {
	int result = 10;

	int i = 0;
	while (i < 10) {
		result = result - 1;
		i++;
	}

	int other = 5;
	for (int j = 0; j < 5; j++) {
		other -= 1;
	}

	return other;
}
