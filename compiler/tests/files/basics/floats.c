float f_val() {
	return 10.0f;
}

int main() {
	float test = f_val();
	int other = 10;

	float result = test - other;

	return (int) result;
}
