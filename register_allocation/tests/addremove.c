void printf(char* content, ...);
int scanf(const char* format, ...);
int* stdin = (int*) 0;
int* stdout = (int*) 0;
int* stderr = (int*) 0;
char* fgets(char *str, int n, int* stream);
void* malloc(unsigned int size);
void free(void* ptr);

int getStudentIndex(int id);
int courseExists(int id);
void printCourses();
int getCourseIndex(int id);
void outputInvoice(int id);

typedef struct {
	int crn;
	char name[10];
	int hours;
} course;

typedef struct {
	int id;
	char name[50];
	course courses[4];
} student;

student *students = (student*) 0;
int studentCount = 0;

course courses[8];
int courseCount = 0;

void addRemoveCourse() {
	// Gets the student
	printf("Enter the student's id: ");

	int id;
	scanf("%d", &id);

	int studentIndex = getStudentIndex(id);
	if (studentIndex == -1) {
		printf("Could not find a student with the ID %d \n", id);
		
		return;
	}

	student *tmpStudent = &students[studentIndex];

	// Displays the students current courses
	printf("Here are the courses [%s] is taking: \n", tmpStudent->name);
	printf("  CRN   Prefix   Cr. Hours \n");
	for (int i = 0; i < 4; i++) {
		course tmpCourse = tmpStudent->courses[i];
		if (courseExists(tmpCourse.crn)) {
			printf("  %4d  %7s  %1d \n", tmpCourse.crn, tmpCourse.name, tmpCourse.hours);
		}
	}

	printf("Choose from: \n");
	printf("  A- Add a new course for [%s] \n", tmpStudent->name);
	printf("  D- Delete a course from [%s]'s schedule \n", tmpStudent->name);
	printf("  C- Cancel oprtation \n");

	printf("Enter your selection: ");

	char input[2];
	fgets(input, 2, stdin);

	while(input[0] == 10) {
		fgets(input, 2, stdin);
	}

	// Checks which option the user selected
	if (input[0] == 'a' || input[0] == 'A') {
		// Checks if the student already has 4 courses (the max amount a student can have)
		int studentsCourses = 0;
		for(int i = 0; i < 4; i++) {
			course tmpCourse = tmpStudent->courses[i];
               		if (courseExists(tmpCourse.crn)) {
        	                studentsCourses++;
	                }
 
		}
		if (studentsCourses == 4) {
			printf("The student already has 4 courses \n");
			
			//input[0] == 10;
			
			return;
		}
		
		// Outputs all available courses
		printf("Available courses: \n");
		printCourses();

		printf("Enter the course number: \n");

		int nCourseID;
		scanf("%d", &nCourseID);

		// Checks if the course exists
		if (!courseExists(nCourseID)) {
			printf("Entered course number does not exist \n");
			
			return;
		}

		// Checks if the student already has this course
		int alreadyTaken = 0;
		for (int i = 0; i < 4; i++) {
			if (tmpStudent->courses[i].crn == nCourseID) {
				alreadyTaken = 1;
				
				break;
			}
		}
		if (alreadyTaken) {
			printf("The entered course is already taken by the student \n");

			return;
		}

		// Searches for a free spot in the course array of the student
		int freeIndex = -1;
		for (int i = 0; i < 4; i++) {
			if (!courseExists(tmpStudent->courses[i].crn)) {
				freeIndex = i;

				break;
			}
		}

		// Adds the course to the student
		int courseIndex = getCourseIndex(nCourseID);

		tmpStudent->courses[freeIndex] = courses[courseIndex];

		printf("Added the course successfully \n");

		
		// Checks if the new invoice should be displayed
		printf("Want to display the new invoice? Y/N: ");

		char tmpIn[2];
		fgets(tmpIn, 2, stdin);

		while(tmpIn[0] == 10) {
			fgets(tmpIn, 2, stdin);
		}

		if (tmpIn[0] == 'y' || tmpIn[0] == 'Y') {
			outputInvoice(tmpStudent->id);
		}
	}

	if (input[0] == 'd' || input[0] == 'D') {
		printf("Enter course number to delete: ");
	
		int courseID;
		scanf("%d", &courseID);

		// Checks if the student actually has the entered course
		int isValid = 0;
		int courseIndex = -1;
		for(int i = 0; i < 4; i++) {
			if (tmpStudent->courses[i].crn == courseID) {
				isValid = 1;
				courseIndex = i;

				break;
			}
		}
		if (!isValid) {
			printf("The student does not have a course with the given ID \n");

			//input[0] == 10;

			return;
		}

		// sets the old course, which should be removed, to a invalid course(invalid course number -10)
		course newCourse;
		
		newCourse.crn = -1;

		tmpStudent->courses[courseIndex] = newCourse;

		printf("Removed the course successfully \n");

		
		// Checks if the new invoice should be displayed 
		printf("Want to display the new invoice? Y/N: ");

        char tmpIn[2];
        fgets(tmpIn, 2, stdin);

        while(tmpIn[0] == 10) {
            fgets(tmpIn, 2, stdin);
        }

        if (tmpIn[0] == 'y' || tmpIn[0] == 'Y') {
            outputInvoice(tmpStudent->id);
        }
	}

	if (input[0] == 'c' || input[0] == 'C') {
		printf("Cancelled the operation \n");
		
		return;
	}

	input[0] = 10;
}