//#include <stdio.h>
//#include <string.h>
//#include <stdlib.h>

void addCourses();

void outputMenu();
int run();

void addStudent();
void addRemoveCourse();
void searchStudent();
void printFee();

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

student *students;
int studentCount;

course courses[8];
int courseCount;

int main() {
	studentCount = 0;

	addCourses();

	while (1) {
		int stopped = run();
		
		printf("\n \n");

		if (stopped) {
			break;
		}
	}

	printf("Goodbye! \n");

	return 0;
}

// adds all the courses to the 'courses' array and sets the 'courseCount' variable
void addCourses() {
	courses[0].crn = 4587;
	strcpy(courses[0].name, "MAT 236");
	courses[0].hours = 4;

	courses[1].crn = 4599;
	strcpy(courses[1].name, "COP 220");
	courses[1].hours = 3;

	courses[2].crn = 8997;
    strcpy(courses[2].name, "GOL 124");
    courses[2].hours = 1;

	courses[3].crn = 9696;
    strcpy(courses[3].name, "COP 100");
    courses[3].hours = 3;

	courses[4].crn = 1232;
    strcpy(courses[4].name, "MAC 531");
    courses[4].hours = 5;

	courses[5].crn = 9856;
    strcpy(courses[5].name, "STA 100");
    courses[5].hours = 2;

	courses[6].crn = 8520;
    strcpy(courses[6].name, "TNV 400");
    courses[6].hours = 5;

	courses[7].crn = 8977;
    strcpy(courses[7].name, "CMP 100");
    courses[7].hours = 1;

	courseCount = 8;
}

// outputs all available options in the main menu
void outputMenu() {
	printf("Choose from the following options: \n");
	printf("  1- Add a new student \n");
	printf("  2- Add/Delete a course \n");
	printf("  3- Search for a student \n");
	printf("  4- Print fee invoice \n");
	printf("  0- Exit program \n");
}

// is called each time one operation is done and its starting again
int run() {
	outputMenu();

	int option;
	scanf("%d", &option);

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

// returns the index, of a student with the given id, in the students array
int getStudentIndex(int id) {
	for(int i = 0; i < studentCount; i++) {
		if (students[i].id == id) {
			return i;
		}
	}

	return -1;
}

// checks whether or not a student with the given id exists
int studentExists(int id) {
	int studentIndex = getStudentIndex(id);
	
	return (studentIndex != -1);
}


// returns the index, of a course with the given id, in the courses array
int getCourseIndex(int id) {
	for (int i = 0; i < courseCount; i++) {
		if (courses[i].crn == id) {
			return i;
		}
	}

	return -1;
}

// checks whether or not a course with the given id exists
int courseExists(int id) {
	int courseIndex = getCourseIndex(id);
	
	return (courseIndex != -1);
}


// prints out all courses
void printCourses() {
	for (int i = 0; i < courseCount; i++) {
		printf("  CRN: %d Name: %s Hours: %d \n", courses[i].crn, courses[i].name, courses[i].hours);
	}
}


// Main function for the 'Add student' Feature
void addStudent() {
	// creates an empty 'student' variable for the new student
	student tmpStudent;
	
	// Gets the new students id, checks if there is already a student with that id and if not sets the id
	printf("Enter the student's id: ");
	
	int id;
	scanf("%d", &id);

	if (studentExists(id)) {
		printf("Student already exists \n");
		
		return;
	}
	tmpStudent.id = id;

	
	// Gets the new students name and automatically sets it
	printf("Enter the student's name: ");

	fgets(tmpStudent.name, 50, stdin);

	while (tmpStudent.name[1] == '\0') {
		fgets(tmpStudent.name, 50, stdin);
	}
	
	
	// Gets how many courses the new student will have 
	printf("Enter how many courses [%s] is taking (up to 4 courses)? \n", tmpStudent.name);

	int courseAmount;
	scanf("%d", &courseAmount);

	if (courseAmount < 0 || courseAmount > 4) {
		printf("Please enter a valid amount of courses \n");

		return;
	}
	
	
	// Displays all available courses
	printf("Available courses: \n");
	printCourses();

	// Gets and adds the courses for the new student
	printf("Enter the [%d] course numbers \n", courseAmount);
	for (int i = 0; i < courseAmount; i++) {
		int courseID;
		scanf("%d", &courseID);

		if (courseExists(courseID)) {
			int courseIndex = getCourseIndex(courseID);

			tmpStudent.courses[i] = courses[courseIndex];
		}else {
			printf("A course with the course number %d does not exist \n", courseID);

			i--;
		}
	}


	// Copys the existing 'students' array into a new array which is one bigger, adds the new student to the end and increases the 'studentCount'
	student *newArray = malloc((studentCount + 1) * sizeof(student));
	for(int i = 0; i < studentCount; i++) {
		newArray[i] = students[i];
	}
	newArray[studentCount] = tmpStudent;

	free(students);
	students = newArray;
	studentCount++;

	printf("Student added Successfully \n");
}

// Outputs the invoice for the student with the given id
void outputInvoice(int studentID) {
	int studentIndex = getStudentIndex(studentID);
	if (studentIndex == -1) {
		printf("Could not find the student");

		return;
	}
	
	student *tmpStudent = &students[studentIndex];
	course *studentCourses = tmpStudent->courses;

	printf("Valence community college \n");
	printf("Orlando FL 10101 \n");
	printf("------------------------ \n");
	printf("\n");
	printf("Fee Invoice Prepared for Student: \n");
	printf("%d-%s", tmpStudent->id, tmpStudent->name);
	printf("\n");
	printf("1 Credit Hour = $120.25 \n");
	printf("\n");
	printf("CRN   CR_PREFIX  CR_HOURS \n");
	float totalPrize = 35.00f;
	for(int i = 0; i < 4; i++) {
		if (courseExists(studentCourses[i].crn)) {
			course tmpCourse = studentCourses[i];
			float prize = tmpCourse.hours * 120.25;
			totalPrize += prize;
			printf("%4d  %7s          %1d      $ %f \n", tmpCourse.crn, tmpCourse.name, tmpCourse.hours, prize);
		}
	}
	printf("            Health & id fees  $  %f \n", 35.00);
	printf("---------------------------------- \n");
	printf("            Total Payments    $ %f \n", totalPrize);
}


// Main function for the 'Add/Remove Course' Feature
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

// Main function for the 'Search Student' Feature
void searchStudent() {
	// Gets a student id and checks if the student exists
	printf("Enter the student's id: ");

    int id;
    scanf("%d", &id);

	int studentIndex = getStudentIndex(id);

	if (studentIndex == -1) {
		printf("No student found \n");
		
		return;
	}

	// Gets the student and outputs some simple data about him/her
	student *tmpStudent = &students[studentIndex];

	printf("ID: %d \n", tmpStudent->id);
	printf("Name: %s \n", tmpStudent->name);
	printf("Courses: \n");
	for(int i = 0; i < 4; i++) {
		course tmpCourse = tmpStudent->courses[i];
		if (courseExists(tmpCourse.crn)) {
			printf("  CRN: %d Name: %s Hours: %d \n", tmpCourse.crn, tmpCourse.name, tmpCourse.hours);
		}
	}
}

// Main function for the 'Fee Print' Feature
void printFee() {
	// Gets the student id and outputs it invoice fee using the function from earlier('outputInvoice')
	printf("Enter the student's id: ");

    int id;
    scanf("%d", &id);

	outputInvoice(id);
}
