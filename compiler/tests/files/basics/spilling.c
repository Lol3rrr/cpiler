void outputMenu();
void scanf(char* t, int* s);
void printf(char* t);
void addStudent();
void addRemoveCourse();
void searchStudent();
void printFee();

int main() {
	outputMenu();

	int option;
	int* opt_ref = &option;
	scanf("%d", opt_ref);

	if (option < 0 || option > 4) {
		printf("Please select one of the options shown above above \n");

		return 0;
	}

	if (option == 1) {
		addStudent();
	}
	if (option == 2) {
		addRemoveCourse();
	}
	if (option == 3) {
		searchStudent();
	}
	if (option == 4) {
		printFee();
	}
	if (option == 0) {
		return 1;
	}

	return 0;
}